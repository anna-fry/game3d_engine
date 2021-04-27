use crate::{geom::{Mat4, Plane, Sphere, Vec3}, shapes::Ball};
const DT: f32 = 1.0 / 60.0;

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
pub struct BallMovement {
    gravity: f32,
}

impl BallMovement {
    pub fn new() -> Self {
        BallMovement {
            gravity: 3.0,
        }
    }
    pub fn update(&self, balls: &mut Vec<Ball>, physics: &mut Vec<Physics>) {
        for (b, p) in balls.iter_mut().zip(physics.iter_mut()) {
            p.momentum += Vec3::new(0.0, -self.gravity, 0.0) * DT;
            let vel = p.momentum / b.mass;
            b.body.c += vel * DT;
        }
    }
}