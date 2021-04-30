use crate::{events::Events, geom::{Mat4, Plane, Sphere, Vec3}, shapes::Ball};
use cgmath::{Vector3, prelude::*};
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
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Force {
    pub force: Vec3
}

impl Force {
    pub fn update_force(&mut self, f: Vec3) {
        self.force += f;
    }

    pub fn new (f: Vec3) -> Force {
        Force {
            force: f
        }
    }

    pub fn apply_force(self) -> Vec3 {
        self.force
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BallMovement {
    gravity: Force,
    player_force: Force,
}

impl BallMovement {
    pub fn new() -> Self {
        BallMovement {
            gravity: Force::new(Vec3::new(0.0, -3.0, 0.0)),
            player_force: Force::new(Vec3::zero()),
        }
    }

    pub fn update(&mut self, events: &Events, balls: &mut Vec<Ball>, physics: &mut Vec<Physics>) {
        match self.player_force.force {
            Vector3 { x, y, z } => {
                
                if events.key_pressed(VirtualKeyCode::Left) {
                    if x > -10.0 {
                        self.player_force.update_force(Vec3::new(-2.0, 0.0, 0.0));
                    }
                }
                else if events.key_pressed(VirtualKeyCode::Right) {
                    if x < 10.0 {
                        self.player_force.update_force(Vec3::new(0.0, 0.0, 2.0));
                    }
                }
                
                if events.key_pressed(VirtualKeyCode::Up) {
                    if y < 10.0 {
                        self.player_force.update_force(Vec3::new(0.0, 0.0, 2.0));
                    }
                }
                else if events.key_pressed(VirtualKeyCode::Down) {
                    if y > -10.0 {
                        self.player_force.update_force(Vec3::new(0.0, 0.0, -2.0));
                    }
                }

                if events.key_pressed(VirtualKeyCode::W) {
                    if z < 10.0 {
                        self.player_force.update_force(Vec3::new(0.0, 2.0, 0.0));
                    }
                }
                else if events.key_pressed(VirtualKeyCode::S) {
                    if z > -10.0 {
                        self.player_force.update_force(Vec3::new(0.0, -2.0, 0.0));
                    }
                }
            }
        }


        

        for (b, p) in balls.iter_mut().zip(physics.iter_mut()) {
            if events.key_pressed(VirtualKeyCode::Space) {
                b.play = true;
            }
            if b.play {
                p.momentum += (b.mass*(self.gravity.apply_force() + self.player_force.apply_force())) * DT;
                let vel = p.momentum / b.mass;
                b.body.c += vel * DT;
            }
        }

        // println!("{:?}", self.player_force)f
    }
}