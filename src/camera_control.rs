use crate::camera::Camera;
use crate::events::{Events};
use crate::shapes::{Ball};
use crate::geom::*;
use cgmath::InnerSpace;
use winit::event::*;

pub struct CameraController {
    pub pitch: f32,
    pub yaw: f32,
    pub player_pos: Pos3,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            player_pos: Pos3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn update(&mut self, events: &Events, player: &mut Ball) {
        // TODO: Change the control to the arrow keys?
        let (dx, dy) = events.mouse_delta();
        self.pitch += dy / 100.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);
        self.yaw -= dx / 100.0;
        self.yaw = self.yaw.clamp(-PI / 4.0, PI / 4.0);
        if !player.play {
            player.body.c = self.player_pos;
        }
    }
    pub fn update_camera(&mut self, c: &mut Camera) {
        c.target = c.eye
            + Quat::new(1.0, 0.0, 0.0, 0.0)
            * Quat::from(cgmath::Euler::new(
                    cgmath::Rad(self.pitch),
                    cgmath::Rad(self.yaw),
                    cgmath::Rad(0.0),
                ))
                * Vec3::unit_z();
        self.player_pos = c.target; 
    }
}