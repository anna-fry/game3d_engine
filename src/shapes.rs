use cgmath::EuclideanSpace;
use cgmath::prelude::*;
use crate::{assets::ModelRef, model::Model, render::InstanceGroups, render::InstanceRaw};
use crate::geom::{Mat4, Plane, Sphere, Vec3};
const DT: f32 = 1.0 / 60.0;

// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct InstanceRaw {
//     #[allow(dead_code)]
//     model: [[f32; 4]; 4],
// }

// impl InstanceRaw {
//     pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
//             // We need to switch from using a step mode of Vertex to Instance
//             // This means that our shaders will only change to use the next
//             // instance when the shader starts processing a new instance
//             step_mode: wgpu::InputStepMode::Instance,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
//                     // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float4,
//                 },
//                 // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
//                 // for each vec4. We don't have to do this in code though.
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
//                     shader_location: 7,
//                     format: wgpu::VertexFormat::Float4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
//                     shader_location: 8,
//                     format: wgpu::VertexFormat::Float4,
//                 },
//             ],
//         }
//     }
// }
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ball {
    pub body: Sphere,
    pub mass: f32,
}

impl Ball {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from_translation(self.body.c.to_vec()) * Mat4::from_scale(self.body.r))
                .into(),
        }
    }

    pub fn render(&self, ball_model: ModelRef, igs: &mut InstanceGroups) {
        igs.render(
            ball_model,
            self.to_raw()
        );
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Static {
    pub body: Plane,
    // control: (i8, i8),
}

impl Static {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from(cgmath::Quaternion::between_vectors(
                Vec3::new(0.0, 1.0, 0.0),
                self.body.n,
            )) * Mat4::from_translation(Vec3::new(0.0, -0.025, 0.0))
                * Mat4::from_nonuniform_scale(0.5, 0.05, 0.5))
            .into(),
        }
    }

    pub fn render(&self, wall_model: ModelRef, igs: &mut InstanceGroups) {
        igs.render(
            wall_model,
            self.to_raw()
        );
    }
}