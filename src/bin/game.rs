use cgmath::prelude::*;
// use game3d_engine::model;
use rand;
use rodio::{Sink, Source, SpatialSink};
use std::{iter, rc::Rc};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::{fs::{self, File}, path::Path};
use std::io::BufReader;
use std::io::prelude::*;

// mod model;
// mod texture;
use game3d_engine::{Engine, Game, audio::Audio, model::{DrawModel, Material, Model, Model2DVertex, ModelVertex, Vertex}, music::Sound, render::InstanceGroups, run, text::Sentence};

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
enum Mode {
    TitleScreen,
    GamePlay,
    EndGame,
}

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
    sounds: Vec<(Sound, bool)>,
    text: Vec<Sentence>,
    text_mat: Rc<Material>,
    menu: (Rect, f32, Rc<Material>),
    // shapes: Vec<Shape>,    // in engine
    // events: Events,        // in engine, inputs from keyboard/keys
    camera: CameraController, // in engine
    mode: Mode
}

impl Components {
    pub fn new(engine: &mut Engine) -> Self {
        let empty_meter = engine.load_material("empty-meter", "content/empty-meter.png");
        let full_meter = engine.load_material("full-meter", "content/full-meter.png");
        let menu = engine.load_material("menu", "content/menu.png");
        let text_mat = engine.load_material("ascii", "content/ascii.png");

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

        let power_text = Sentence::text_to_sentence("Power", [-0.8, -0.55]);
        let text = vec![power_text];

        let camera = CameraController::new();
        
        let collide_sound = Sound::load("content/ball_collide.mp3").unwrap();
        
        let sounds = vec![(collide_sound, true)];
        Components {
            balls: balls,
            statics: walls,
            goal: goal,
            meter: meter,
            physics: physics,
            models: game_data,
            score: 0,
            sounds: sounds,
            text: text,
            text_mat: text_mat,
            menu: (Rect { x: -0.9, y: -0.9, w: 1.8, h: 1.8 }, 1.0, menu),
            camera: camera,
            mode: Mode::TitleScreen
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
    pub fn process(&mut self, events: &Events, c: &mut Components, sink: &Audio) {
        self.ball_movement
            .update(events, &mut c.balls, &mut c.meter[1], &mut c.physics);
        let effect =
            self.collision_detection
                .update(&c.statics, &mut c.balls, &c.goal, &mut c.physics);

        // println!("effect: {:?}", effect);
        match effect {
            game3d_engine::collision::CollisionEffect::Score => {
                c.score += 1;
                c.balls[0].play = false;
                self.ball_movement.player_mag = 0.0;
                c.physics[0].reset();
                c.meter[1].0.w = 0.0;
                c.meter[1].1 = 0.0;
                c.goal.gen_new_loc();
                c.sounds[0].1 = true;
            },
            game3d_engine::collision::CollisionEffect::WallCollision => {
                // println!("make it here");
                if c.sounds[0].1 {

                    let pos = c.balls[0].body.c;
                    // c.sink.set_volume(2.0);
                    // sink.set_emitter_position([pos.x, pos.y, pos.z]);
                    // sink.append(c.sounds[0].0.decoder());
                    sink.play(pos, c.sounds[0].0.clone());
                    c.sounds[0].1 = false;
                }

                // println!("make it past append");
            }
            
            
            _ => {}
        }
        if events.key_released(VirtualKeyCode::Return) {
            c.balls[0].play = false;
            self.ball_movement.player_mag = 0.0;
            c.physics[0].reset();
            c.meter[1].0.w = 0.0;
            c.meter[1].1 = 0.0;
            c.sounds[0].1 = true;
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
        match self.components.mode{
            Mode::TitleScreen => {
                //TODO: make a title page; can still press return tho
                if engine.events.key_pressed(VirtualKeyCode::Return) == true {
                    self.components.mode = Mode::GamePlay;
                } else if engine.events.key_pressed(VirtualKeyCode::L){
                    load_game(&mut self.components);
                    self.components.mode = Mode::GamePlay;
                }
            }
            Mode::GamePlay => {
                if engine.events.key_held(VirtualKeyCode::P){
                    save_game(&mut self.components);
                }
                self.components
                    .camera
                    .update(&engine.events, &mut self.components.balls[0]);
                self.components.camera.update_camera(engine.camera_mut());
                self.systems.process(&engine.events, &mut self.components, &engine.sink);
            }
            Mode::EndGame => {}
        }
        
        
    }

    fn render(&self, igs: &mut InstanceGroups) {
        match self.components.mode {
            Mode::TitleScreen => {             
                igs.render_bar(&self.components.menu.0, self.components.menu.1, &self.components.menu.2);
            },
            Mode::GamePlay => {
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
        
                for sentence in self.components.text.iter() {
                    sentence.draw_sentence(igs, &self.components.text_mat);
                }
        
                let score_sentence = Sentence::text_to_sentence(&("Score: ".to_string() + &self.components.score.to_string()), [-0.2, 0.9]);
                score_sentence.draw_sentence(igs, &self.components.text_mat);
            },
            Mode::EndGame => ()
        }
    }
}

pub fn save_game(components: &mut Components) -> std::io::Result<()>{
let serialized = serde_json::to_string(&components.score).unwrap();
    fs::write("saved.txt", serialized);

    let file = File::open("saved.txt")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(())
}
pub fn load_game(components: &mut Components) -> std::io::Result<()>{
    if Path::new("saved.txt").exists(){
        let file = File::open("saved.txt")?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        let deserialized: usize = serde_json::from_str(&contents).unwrap();
        components.score = deserialized;
    }
    //include a message that there was not saved gamestate
    Ok(())
    
}


fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    game3d_engine::run::<Components, Systems, BallGame>(window, std::path::Path::new("content"));
}
