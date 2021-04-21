use cgmath::prelude::*;
use rand;
use std::{iter, path::Path};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub mod model;
pub mod texture;
pub mod geom;
use model::{DrawModel, Vertex};
pub mod shapes;
use shapes::*;
pub mod camera;
use camera::Camera;
pub mod camera_control;
use camera_control::CameraController;

pub mod collision;


pub const DT: f32 = 1.0 / 60.0;

pub trait Game: Sized {
    type StaticData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData);
    fn update(&mut self, rules: &Self::StaticData, engine: &mut Engine);
    fn render(&self, rules: &Self::StaticData, igs: &mut InstanceGroups);
}

pub struct Engine {
    pub frame: usize,
    // pub assets: Assets,
    // render: Render,
    // pub events: Events,
}

// impl Engine {
//     pub fn load_model(&mut self, model: impl AsRef<Path>) -> assets::ModelRef {
//         self.assets.load_model(
//             &self.render.device,
//             &self.render.queue,
//             &self.render.texture_layout,
//             model,
//         )
//     }
//     pub fn camera_mut(&mut self) -> &mut camera::Camera {
//         &mut self.render.camera
//     }
//     pub fn set_ambient(&mut self, amb: f32) {
//         self.render.set_ambient(amb);
//     }
//     pub fn set_lights(&mut self, lights: impl IntoIterator<Item = lights::Light>) {
//         self.render.set_lights(lights.into_iter().collect());
//     }
// }

pub fn run<State>(
    mut state: State,
    window_builder: winit::window::WindowBuilder,
    asset_root: &Path,
) {
   
    use std::time::Instant;
    env_logger::init();
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    use futures::executor::block_on;
    

    // How many frames have we simulated?
    #[allow(unused_variables)]
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time: f32 = 0.0;
    let mut since = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
                // The renderer "produces" time...
                available_time += since.elapsed().as_secs_f32();
                since = Instant::now();
            }
            _ => {}
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            state.update();

            // Increment the frame counter
            frame_count += 1;
        }
    });
}
