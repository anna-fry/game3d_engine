use cgmath::EuclideanSpace;
use cgmath::prelude::*;
use rand::Rng;
use crate::{assets::ModelRef, render::InstanceGroups, render::InstanceRaw};
use crate::geom::{Mat4, Plane, Sphere, Vec3, Box, Pos3};



#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ball {
    pub body: Sphere,
    pub pitch: f32,
    pub yaw: f32,
    pub mass: f32,
    pub play: bool,
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
    pub position: Vec3
    // control: (i8, i8),
}

impl Static {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: ( Mat4::from(cgmath::Quaternion::between_vectors(
                Vec3::new(0.0, 1.0, 0.0),
                self.body.n,
            ))
            * Mat4::from_translation(self.position)
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

pub struct Goal {
    pub body: Box,
}

impl Goal {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Mat4::from_translation(self.body.c.to_vec()) * Mat4::from_nonuniform_scale(self.body.r[0], self.body.r[1], self.body.r[2]))
            .into(),
        }
    }
    
    pub fn render(&self, goal_model: ModelRef, igs: &mut InstanceGroups) {
        igs.render(
            goal_model,
            self.to_raw()
        );
    }

    pub fn gen_new_loc(&mut self) {
        let mut rng = rand::thread_rng();
        let new_x = rng.gen_range(-12.5..-0.5);
        let new_y = rng.gen_range(0.5..7.0);
        let new_z = rng.gen_range(-12.5..-0.5);
        self.body.c = Pos3::new(new_x, new_y, new_z);
    }
}