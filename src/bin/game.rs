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
use game3d_engine::{Engine, Game, model::{DrawModel, Model, ModelVertex, Vertex}, render::InstanceGroups, run};

use game3d_engine::texture::*;

use game3d_engine::shapes::{Ball, Static};
// mod camera;
use game3d_engine::camera::Camera;
// mod camera_control;
use game3d_engine::camera_control::CameraController;

use game3d_engine::geom::*;
// mod collision;
use game3d_engine::collision::CollisionDetection;

use game3d_engine::physics::{Physics, BallMovement};

use game3d_engine::events::{Events};

// use game3d_engine::render::{Render};

struct GameData {
    ball_model: game3d_engine::assets::ModelRef,
    wall_model: game3d_engine::assets::ModelRef,
}

pub struct Components {
    balls: Vec<Ball>,      // game specific
    statics: Vec<Static>,  // game specific
    // goal: Vec<Goal>,       // game specific
    physics: Vec<Physics>, // in engine
    models: GameData,    // in engine
    // shapes: Vec<Shape>,    // in engine
    // events: Events,        // in engine, inputs from keyboard/keys
    //camera: Camera,        // in engine
}

impl Components {
    pub fn new(engine: &mut Engine) -> Self{
        let balls = vec![
            Ball {
                body: Sphere {
                    c: Pos3::new(0.0, 3.0, 0.0),
                    r: 0.3,
                },
                mass: 0.12
            },
        ];
        let statics = vec![
            Static {
                body: Plane {
                    n: Vec3::new(0.0, 1.0, 0.0),
                    d: 0.0,
                },
                position: Vec3::new(0.0, -0.025, 0.0)
            },
            Static {
                body: Plane {
                    n: Vec3::new(0.0, 0.0, 1.0),
                    d: 0.0,
                },
                position: Vec3::new(0.0, 0.0, 20.0)
            },
            Static {
                body: Plane {
                    n: Vec3::new(-1.0, 0.0, -1.0),
                    d: 0.0,
                },
                position: Vec3::new(20.0, 0.0, -20.0)
            }
        ];
        let physics = vec![
            Physics {
                velocity: Vec3::zero(),
                momentum: Vec3::zero(),
                force: Vec3::zero(),
            }
        ];
        let game_data = GameData {
            ball_model: engine.load_model("sphere.obj"),
            wall_model: engine.load_model("floor.obj")
        };
        Components {
            balls: balls,
            statics: statics,
            physics: physics,
            models: game_data,
        }
    }
}


pub struct Systems {
    ball_movement: BallMovement,             // game specific
    collision_detection: CollisionDetection, // in engine
    // render: Render,                          // in engine
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            ball_movement: BallMovement::new(),
            collision_detection: CollisionDetection::new(),
        }
    }
    pub fn process(&mut self, c: &mut Components) {
        self.ball_movement.update(&mut c.balls, &mut c.physics);
        self.collision_detection.update(&c.statics, &mut c.balls, &mut c.physics);
    }
}

pub struct BallGame {
    components: Components,
    systems: Systems,
}

impl Game for BallGame {
    type StaticData = Components;
    type SystemData = Systems;
    fn start(engine: &mut Engine) -> Self { 
        let components = Components::new(engine);
        let systems = Systems::new();
        let game = BallGame {
            components: components,
            systems: systems,
        };
        game
    }

    fn update(&mut self, engine: &mut Engine) {
        self.systems.process(&mut self.components);
    }

    fn render(&self, igs: &mut InstanceGroups) {
        for ball in  self.components.balls.iter() {
            ball.render(self.components.models.ball_model, igs);
        }

        for stat in self.components.statics.iter() {
            stat.render(self.components.models.wall_model, igs);
        }
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    game3d_engine::run::<Components, Systems, BallGame>(window, std::path::Path::new("content"));
}
