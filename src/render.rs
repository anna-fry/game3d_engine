use crate::camera::Camera;
use crate::model::*;
use crate::texture;
use crate::Game;
use crate::{
    assets::{Assets, ModelRef},
    geom::Rect,
};
use cgmath::SquareMatrix;
use std::{collections::BTreeMap, rc::Rc};
use wgpu::util::DeviceExt;

use winit::window::Window;
pub(crate) struct Render {
    surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    render_3d_pipeline: wgpu::RenderPipeline,
    render_2d_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_layout: wgpu::BindGroupLayout,
    pub(crate) camera: Camera,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    instance_groups: InstanceGroups,
}

impl Render {
    pub(crate) async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera = Camera {
            eye: (-15.0, 5.0, -15.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_2d_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render 2D Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let vs_2d_module = device.create_shader_module(&wgpu::include_spirv!("shader_2d.vert.spv"));
        let fs_2d_module = device.create_shader_module(&wgpu::include_spirv!("shader_2d.frag.spv"));

        let render_3d_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let render_2d_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&render_2d_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_2d_module,
                entry_point: "main",
                buffers: &[Model2DVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_2d_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            instance_groups: InstanceGroups::new(&device),
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_3d_pipeline,
            render_2d_pipeline,
            camera,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            texture_layout: texture_bind_group_layout,
            depth_texture,
        }
    }

    pub(crate) fn update_buffers<R, G: Game<StaticData = R>>(
        &mut self,
        game: &G,
        assets: &mut Assets,
    ) {
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
        self.instance_groups.clear();
        game.render(&mut self.instance_groups);
        self.instance_groups
            .update_buffers(&self.queue, &self.device, assets);
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    pub(crate) fn render<R, G: Game<StaticData = R>>(
        &mut self,
        game: &G,
        assets: &mut Assets,
    ) -> Result<(), wgpu::SwapChainError> {
        self.update_buffers(game, assets);

        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_3d_pipeline);
            for (mr, (irs, buf, _cap)) in self.instance_groups.groups.iter() {
                render_pass.set_vertex_buffer(1, buf.as_ref().unwrap().slice(..));
                render_pass.draw_model_instanced(
                    assets.get_model(*mr).unwrap(),
                    0..irs.len() as u32,
                    &self.uniform_bind_group,
                );
            }

            render_pass.set_pipeline(&self.render_2d_pipeline);

            for (_rect, _power, mat, buffer) in self.instance_groups.groups_2d.iter() {
                // rect into four model 2d vertices

                // println!("{:?}", rect_vertices);
                // self.queue.write_buffer(quad_buffer, 0, vertices);
                render_pass.set_vertex_buffer(0, buffer.as_ref().unwrap().slice(..));
                render_pass.set_bind_group(0, &mat.bind_group, &[]);
                render_pass.draw(0..4, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

pub struct InstanceGroups {
    groups: BTreeMap<ModelRef, (Vec<InstanceRaw>, Option<wgpu::Buffer>, usize)>,
    groups_2d: Vec<(Rect, Rect, Rc<Material>, Option<wgpu::Buffer>)>,
}
impl InstanceGroups {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            groups: BTreeMap::new(),
            groups_2d: vec![],
        }
    }
    fn clear(&mut self) {
        for (_mr, (irs, _buf, _cap)) in self.groups.iter_mut() {
            irs.clear();
        }

        self.groups_2d.clear();
    }
    fn update_buffers(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, assets: &Assets) {
        for (mr, (irs, buf, cap)) in self.groups.iter_mut() {
            if buf.is_none() || *cap < irs.len() {
                buf.replace(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(assets.path_for_model_ref(*mr).to_str().unwrap()),
                        usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                        contents: bytemuck::cast_slice(irs),
                    }),
                );
                *cap = irs.len();
            } else {
                queue.write_buffer(buf.as_ref().unwrap(), 0, bytemuck::cast_slice(irs));
            }
        }
        // Model2DVertex {
        //     position: [rect.x, rect.y + rect.h],
        //     tex_coords: [0.0, 0.0],
        // },
        // Model2DVertex {
        //     position: [rect.x, rect.y],
        //     tex_coords: [0.0, 1.0],
        // },
        // Model2DVertex {
        //     position: [rect.x + rect.w, rect.y + rect.h],
        //     tex_coords: [*power, 0.0],
        // },
        // Model2DVertex {
        //     position: [rect.x + rect.w, rect.y],
        //     tex_coords: [*power, 1.0],
        // },
        for (rect, tex_rect, mat, buffer) in self.groups_2d.iter_mut() {
            let rect_vertices = [
                Model2DVertex {
                    position: [rect.x, rect.y + rect.h],
                    tex_coords: [tex_rect.x, tex_rect.y],
                },
                Model2DVertex {
                    position: [rect.x, rect.y],
                    tex_coords: [tex_rect.x, tex_rect.y + tex_rect.h],
                },
                Model2DVertex {
                    position: [rect.x + rect.w, rect.y + rect.h],
                    tex_coords: [tex_rect.x + tex_rect.w, tex_rect.y],
                },
                Model2DVertex {
                    position: [rect.x + rect.w, rect.y],
                    tex_coords: [tex_rect.x + tex_rect.w, tex_rect.y + tex_rect.h],
                },
            ];
            let vertices = bytemuck::cast_slice(&rect_vertices);
            if buffer.is_none() {
                buffer.replace(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("2d buffer"),
                        usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                        contents: vertices,
                    }),
                );
            } else {
                queue.write_buffer(buffer.as_ref().unwrap(), 0, vertices);
            }
        }
    }
    pub fn render(&mut self, mr: ModelRef, ir: InstanceRaw) {
        self.render_batch(mr, std::iter::once(ir));
    }
    pub fn render_batch(&mut self, mr: ModelRef, ir: impl IntoIterator<Item = InstanceRaw>) {
        let ref mut groups = self.groups;
        groups
            .entry(mr)
            .or_insert((vec![], None, 0))
            .0
            .extend(ir.into_iter())
    }

    pub fn render_bar(&mut self, rect: &Rect, power: f32, mat: &Rc<Material>) {
        let tex_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: power,
            h: 1.0
        };

        self.groups_2d.push((*rect, tex_rect, Rc::clone(mat), None));
    }

    pub fn render_2d(&mut self, rect: &Rect, tex_rect: &Rect, mat: &Rc<Material>) {
        self.groups_2d.push((*rect, *tex_rect, Rc::clone(mat), None));
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}
