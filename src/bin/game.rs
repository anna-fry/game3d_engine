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

struct Systems {
    ball_movement: BallMovement,             // game specific
    collision_detection: CollisionDetection, // in engine
    render: Render,                          // in engine
}

impl Systems {
    pub fn process(g: &mut Game) {
        // call each system's process function
    }
}

struct Game {
    components: Components,
    systems: Systems,
}

impl Game {
    fn new() {
        // Initialize components and systems
    }
}

fn main() {
    let game: engine::Game<Components, Systems> = Game::new();
    run(game);
}
