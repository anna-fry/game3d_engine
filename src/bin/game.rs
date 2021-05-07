use cgmath::prelude::*;
// use game3d_engine::model;
use rand;
use std::{iter, rc::Rc};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// mod model;
// mod texture;
use game3d_engine::{
    model::{DrawModel, Material, Model, Model2DVertex, ModelVertex, Vertex},
    render::InstanceGroups,
    run, Engine, Game,
};

use game3d_engine::texture::*;

use game3d_engine::shapes::{Ball, Goal, Static};
// mod camera;
use game3d_engine::camera::Camera;
// mod camera_control;
use game3d_engine::camera_control::CameraController;

use game3d_engine::geom::*;
// mod collision;
use game3d_engine::collision::{CollisionDetection, CollisionEffect};

use game3d_engine::physics::{BallMovement, Physics};

use game3d_engine::events::Events;

use winit::event::VirtualKeyCode;

// use game3d_engine::render::{Render};

struct GameData {
    ball_model: game3d_engine::assets::ModelRef,
    wall_model: game3d_engine::assets::ModelRef,
    floor_model: game3d_engine::assets::ModelRef,
    goal_model: game3d_engine::assets::ModelRef,
}

pub struct Components {
    balls: Vec<Ball>,     // game specific
    statics: Vec<Static>, // game specific
    goal: Goal,           // game specific
    meter: Vec<(Rect, f32, Rc<Material>)>,
    physics: Vec<Physics>, // in engine
    models: GameData,      // in engine
    score: usize,
    // shapes: Vec<Shape>,    // in engine
    // events: Events,        // in engine, inputs from keyboard/keys
    camera: CameraController, // in engine
}

impl Components {
    pub fn new(engine: &mut Engine) -> Self {
        let empty_meter = engine.load_material("empty-meter", "content/empty-meter.png");
        let full_meter = engine.load_material("full-meter", "content/full-meter.png");

        let meter = vec![
            (
                Rect {
                    x: -0.8,
                    y: -0.8,
                    w: 0.6,
                    h: 0.2,
                },
                1.0,
                empty_meter,
            ),
            (
                Rect {
                    x: -0.8,
                    y: -0.8,
                    w: 0.0,
                    h: 0.2,
                },
                0.0,
                full_meter,
            ),
        ];

        let balls = vec![Ball {
            body: Sphere {
                c: Pos3::new(-20.0, 5.0, -20.0),
                r: 0.1,
            },
            pitch: 0.0,
            yaw: 0.0,
            mass: 4.0 * 3.14 * (0.3_f32).powf(3.0) / 3.0,
            play: false,
        }];

        let walls = vec![
            Static {
                body: Plane {
                    n: Vec3::new(0.0, 1.0, 0.0),
                    d: 2.0,
                },
                position: Vec3::new(0.0, -0.025, 0.0),
            },
            Static {
                body: Plane {
                    n: Vec3::new(0.0, 0.0, -1.0),
                    d: 2.0,
                },
                position: Vec3::new(0.0, -0.025, 0.0),
            },
            Static {
                body: Plane {
                    n: Vec3::new(-1.0, 0.0, 0.0),
                    d: 2.0,
                },
                position: Vec3::new(0.0, -0.025, 0.0),
            },
        ];
        let goal = Goal {
            body: Box {
                c: Pos3::new(-2.0, 1.5, -3.0),
                r: Pos3::new(0.5, 0.5, 0.5),
            },
        };
        let physics = vec![Physics {
            velocity: Vec3::zero(),
            momentum: Vec3::zero(),
            force: Vec3::zero(),
        }];
        let game_data = GameData {
            ball_model: engine.load_model("sphere.obj"),
            wall_model: engine.load_model("wall.obj"),
            floor_model: engine.load_model("floor.obj"),
            goal_model: engine.load_model("dustbin.obj"),
        };
        let camera = CameraController::new();
        Components {
            balls: balls,
            statics: walls,
            goal: goal,
            meter: meter,
            physics: physics,
            models: game_data,
            score: 0,
            camera: camera,
        }
    }
}

pub struct Systems {
    ball_movement: BallMovement, // game specific
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
    pub fn process(&mut self, events: &Events, c: &mut Components) {
        self.ball_movement
            .update(events, &mut c.balls, &mut c.meter[1], &mut c.physics);
        let effect =
            self.collision_detection
                .update(&c.statics, &mut c.balls, &c.goal, &mut c.physics);
        match effect {
            game3d_engine::collision::CollisionEffect::Score => {
                c.score += 1;
                c.balls[0].play = false;
                self.ball_movement.player_mag = 0.0;
                c.physics[0].reset();
                c.meter[1].0.w = 0.0;
                c.meter[1].1 = 0.0;
            }
            _ => (),
        }
        if events.key_released(VirtualKeyCode::Return) {
            c.balls[0].play = false;
            self.ball_movement.player_mag = 0.0;
            c.physics[0].reset();
            c.meter[1].0.w = 0.0;
            c.meter[1].1 = 0.0;
        }
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
        self.components
            .camera
            .update(&engine.events, &mut self.components.balls[0]);
        self.components.camera.update_camera(engine.camera_mut());
        self.systems.process(&engine.events, &mut self.components);
    }

    fn render(&self, igs: &mut InstanceGroups) {
        for ball in self.components.balls.iter() {
            ball.render(self.components.models.ball_model, igs);
        }

        for stat in self.components.statics.iter() {
            //I just picked the floor value that was different from the rest
            if stat.body.n.y == 1.0 {
                stat.render(self.components.models.floor_model, igs);
            } else {
                stat.render(self.components.models.wall_model, igs);
            }
        }

        self.components
            .goal
            .render(self.components.models.goal_model, igs);


        for (rect, power, mat) in self.components.meter.iter() {
            igs.render_bar(&rect, *power, mat);
        }
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    game3d_engine::run::<Components, Systems, BallGame>(window, std::path::Path::new("content"));
}
