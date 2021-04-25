use cgmath::prelude::*;
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

use game3d_engine::shapes::{InstanceRaw, Ball, Static};
// mod camera;
use game3d_engine::camera::Camera;
// mod camera_control;
use game3d_engine::camera_control::CameraController;

use game3d_engine::geom::*;
// mod collision;
use game3d_engine::collision::*;

use game3d_engine::physics::{Physics, BallMovement};

struct Components {
    balls: Vec<Ball>,      // game specific
    statics: Vec<Static>,  // game specific
    goal: Vec<Goal>,       // game specific
    physics: Vec<Physics>, // in engine
    models: Vec<Model>,    // in engine
    shapes: Vec<Shape>,    // in engine
    events: Events,        // in engine, inputs from keyboard/keys
    camera: Camera,        // in engine
}

impl Components {
    pub fn new() {

    }
}

struct Systems {
    ball_movement: BallMovement,             // game specific
    collision_detection: CollisionDetection, // in engine
    render: Render,                          // in engine
}

impl Systems {
    pub fn new() {

    }
    pub fn process(&mut self, g: &mut Game) {
        self.ball_movement.update(&mut g.balls, &mut g.physics);
        self.collision_detection.update(&g.statics, &mut g.balls, &mut g.physics);
    }
}

struct BallGame {
    components: Components,
    systems: Systems,
}

impl Game for BallGame {
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {

    }
    fn update(&mut self, rules: &Self::StaticData, engine: &mut Engine) {

    }
    fn render(&self, rules: &Self::StaticData, igs: &mut InstanceGroups) {

    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    run::<Components, Systems, Game<Components, Systems>>(window, std::path::Path::new("content"));
}
