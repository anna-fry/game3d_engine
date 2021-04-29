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
    pub fn update(&mut self, events: &Events, player: &Ball) {
        // TODO: Change the control to the arrow keys?
        let (dx, dy) = events.mouse_delta();
        self.pitch += dy / 100.0;
        self.pitch = self.pitch.clamp(-PI / 4.0, PI / 4.0);
        self.yaw -= dx / 100.0;
        self.yaw = self.yaw.clamp(-PI / 4.0, PI / 4.0);
        // TODO: Figure out how to keep the primary ball right in front of the camera
        // self.player_pos = player.body.c;
    }
    pub fn update_camera(&self, c: &mut Camera) {
        // TODO: Figure out how to keep the player just in front of the camera
        // c.eye = self.player_pos + Vec3::new(0.0, 0.5, 0.0);
        // The camera is pointing at a point just in front of the composition of the player's rot and the camera's rot (player * cam * forward-offset)
        c.target = c.eye
            + Quat::new(1.0, 0.0, 0.0, 0.0)
            * Quat::from(cgmath::Euler::new(
                    cgmath::Rad(self.pitch),
                    cgmath::Rad(self.yaw),
                    cgmath::Rad(0.0),
                ))
                * Vec3::unit_z();
    }
}