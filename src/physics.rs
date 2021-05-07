use std::rc::Rc;

use crate::{events::Events, geom::{Mat4, Plane, Rect, Sphere, Vec3}, model::Material, shapes::Ball};
use cgmath::{prelude::*, Vector3};
use winit::event::VirtualKeyCode;
pub const DT: f32 = 1.0 / 60.0;

pub struct Physics {
    pub velocity: Vec3,
    pub momentum: Vec3,
    pub force: Vec3,
}

impl Physics {
    pub fn apply_impulse(&mut self, f: Vec3) {
        self.momentum += f;
    }
    pub fn reset(&mut self) {
        self.velocity = Vec3::zero();
        self.momentum = Vec3::zero();
        self.force = Vec3::zero();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Force {
    pub force: Vec3,
}

impl Force {
    pub fn update_force(&mut self, f: Vec3) {
        self.force += f;
    }

    pub fn new(f: Vec3) -> Force {
        Force { force: f }
    }

    pub fn apply_force(self) -> Vec3 {
        self.force
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BallMovement {
    gravity: Force,
    pub player_mag: f32,
}

impl BallMovement {
    pub fn new() -> Self {
        BallMovement {
            gravity: Force::new(Vec3::new(0.0, -3.0, 0.0)),
            player_mag: 0.0,
        }
    }

    pub fn update(&mut self, events: &Events, balls: &mut Vec<Ball>, meter: &mut (Rect, f32, Rc<Material>), physics: &mut Vec<Physics>) {
        if events.key_held(VirtualKeyCode::Up) {
            if self.player_mag < 40.0 {
                self.player_mag += 2.0;
                meter.0.w += 0.03;
                meter.1 += 0.05;
            }
        } else if events.key_held(VirtualKeyCode::Down) {
            if self.player_mag > 0.0 {
                self.player_mag -= 2.0;
                meter.0.w -= 0.03;
                meter.1 -= 0.05;
            }
        }

        for (b, p) in balls.iter_mut().zip(physics.iter_mut()) {
            if events.key_pressed(VirtualKeyCode::Space) {
                b.play = true;
                let x = self.player_mag * b.yaw.sin() * (-b.pitch).cos();
                let y = self.player_mag * b.yaw.sin() * (-b.pitch).sin();
                let z = self.player_mag * b.yaw.cos();
                let player_force = Force::new(Vec3::new(x, y, z));
                p.momentum += player_force.apply_force() * DT;
            }
            if b.play {
                p.momentum += (b.mass * self.gravity.apply_force()) * DT;
                let vel = p.momentum / b.mass;
                b.body.c += vel * DT;
            }
        }

        // println!("{:?}", self.player_force)f
    }
}
