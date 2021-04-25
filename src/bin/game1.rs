/*use cgmath::prelude::*;
// use game3d_engine::model;
use rand;
use std::iter;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// mod model;
// mod texture;
use game3d_engine::model::{DrawModel, Model, ModelVertex, Vertex};

use game3d_engine::texture::*;

use game3d_engine::shapes::{InstanceRaw, Marble, Wall};
// mod camera;
use game3d_engine::camera::Camera;
// mod camera_control;
use game3d_engine::camera_control::CameraController;

use game3d_engine::geom::*;
// mod collision;
use game3d_engine::collision::*;

// const NUM_MARBLES: usize = 50;

const DT: f32 = 1.0 / 60.0;

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

// mod geom;
// use geom::*;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    marble_model: Model,
    wall_model: Model,
    camera: Camera,
    camera_controller: CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    marble: Marble,
    walls: Vec<Wall>,
    g: f32,
    #[allow(dead_code)]
    marbles_buffer: wgpu::Buffer,
    walls_buffer: wgpu::Buffer,
    depth_texture: Texture,
    contacts: Contacts,
}

impl State {
    async fn new(window: &Window) -> Self {
        use rand::Rng;
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
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };
        let camera_controller = CameraController::new(0.2);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let walls = vec![
            Wall {
                body: Plane {
                    n: Vec3::new(0.0, 1.0, 0.0),
                    d: 0.0,
                },
            },
            Wall {
                body: Plane {
                    n: Vec3::new(1.0, 0.0, 0.0),
                    d: 0.0,
                },
            },
        ];
        let mut rng = rand::thread_rng();
        let r = 1.0;
        let marble = Marble {
            body: Sphere {
                c: Pos3::new(0.0, 0.0, 0.0),
                r: r,
            },
            velocity: Vec3::zero(),
            mass: 4.0 * 3.14 * r.powf(3.0) / 3.0,
            momentum: Vec3::zero(),
            force: Vec3::zero(),
        };

        let marbles_data = vec![marble.to_raw()];
        let marbles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Marbles Buffer"),
            contents: bytemuck::cast_slice(&marbles_data),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });
        let wall_data = walls.iter().map(Wall::to_raw).collect::<Vec<_>>();
        let walls_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Walls Buffer"),
            contents: bytemuck::cast_slice(&wall_data),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
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

        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("content");
        let wall_model = Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            res_dir.join("floor.obj"),
        )
        .unwrap();
        let marble_model = Model::load(
            &device,
            &queue,
            &texture_bind_group_layout, // It's shaded the same as the floor
            res_dir.join("sphere.obj"),
        )
        .unwrap();

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../shader.frag.spv"));

        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                format: Texture::DEPTH_FORMAT,
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

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            marble_model,
            wall_model,
            camera,
            camera_controller,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            marble,
            walls,
            g: 3.0,
            marbles_buffer,
            walls_buffer,
            depth_texture,
            contacts: Contacts::new(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event) //|| self.wall.process_events(event)
    }

    fn update(&mut self) {
        use rand::Rng;
        self.camera_controller.update_camera(&mut self.camera);
        // we ~could~ move the plane, or we could just tweak gravity.
        // this time we'll move the plane.
        self.marble.update(self.g);
        if (self.marble.body.c.distance(Pos3::new(0.0, 0.0, 0.0))) >= 40.0 {
            self.marble.body = Sphere {
                c: Pos3::new(0.0, 0.0, 0.0),
                r: r,
            };
            self.marble.velocity = Vec3::zero();
            self.marble.momentum = Vec3::zero();
            self.marble.force = Vec3::zero();
        }
        self.marble.update(self.g);

        update(&self.walls, &mut vec![self.marble], &mut self.contacts);
        for Contact { a: ma, .. } in self.contacts.wm.iter() {
            // apply "friction" to marbles on the ground
            self.marble.momentum *= 0.995;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // Update buffers based on dynamics
        self.queue.write_buffer(
            &self.walls_buffer,
            0,
            bytemuck::cast_slice(&vec![self.wall.to_raw()]),
        );
        // TODO avoid reallocating every frame
        let marbles_data = vec![self.marble.to_raw()];
        self.queue
            .write_buffer(&self.marbles_buffer, 0, bytemuck::cast_slice(&marbles_data));
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

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

            render_pass.set_vertex_buffer(1, self.marbles_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(&self.marble_model, 0..1, &self.uniform_bind_group);
            render_pass.set_vertex_buffer(1, self.walls_buffer.slice(..));
            render_pass.draw_model_instanced(
                &self.wall_model,
                0..self.walls.len() as u32,
                &self.uniform_bind_group,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

// fn main() {

//     use std::time::Instant;
//     env_logger::init();
//     let event_loop = EventLoop::new();
//     let title = env!("CARGO_PKG_NAME");
//     let window = winit::window::WindowBuilder::new()
//         .with_title(title)
//         .build(&event_loop)
//         .unwrap();
//     use futures::executor::block_on;
//     let mut state = block_on(State::new(&window));

//     // How many frames have we simulated?
//     #[allow(unused_variables)]
//     let mut frame_count: usize = 0;
//     // How many unsimulated frames have we saved up?
//     let mut available_time: f32 = 0.0;
//     let mut since = Instant::now();

//     event_loop.run(move |event, _, control_flow| {
//         *control_flow = ControlFlow::Poll;
//         match event {
//             Event::MainEventsCleared => window.request_redraw(),
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == window.id() => {
//                 if !state.input(event) {
//                     match event {
//                         WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                         WindowEvent::KeyboardInput { input, .. } => match input {
//                             KeyboardInput {
//                                 state: ElementState::Pressed,
//                                 virtual_keycode: Some(VirtualKeyCode::Escape),
//                                 ..
//                             } => {
//                                 *control_flow = ControlFlow::Exit;
//                             }
//                             _ => {}
//                         },
//                         WindowEvent::Resized(physical_size) => {
//                             state.resize(*physical_size);
//                         }
//                         WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                             state.resize(**new_inner_size);
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//             Event::RedrawRequested(_) => {
//                 state.update();
//                 match state.render() {
//                     Ok(_) => {}
//                     // Recreate the swap_chain if lost
//                     Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
//                     // The system is out of memory, we should probably quit
//                     Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                     // All other errors (Outdated, Timeout) should be resolved by the next frame
//                     Err(e) => eprintln!("{:?}", e),
//                 }
//                 // The renderer "produces" time...
//                 available_time += since.elapsed().as_secs_f32();
//                 since = Instant::now();
//             }
//             _ => {}
//         }
//         // And the simulation "consumes" it
//         while available_time >= DT {
//             // Eat up one frame worth of time
//             available_time -= DT;

//             state.update();

//             // Increment the frame counter
//             frame_count += 1;
//         }
//     });
// }

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    let game_state = run::<State>();
}
*/