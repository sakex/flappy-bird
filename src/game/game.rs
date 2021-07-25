use crate::game::bird::Bird;
use crate::game::pipe::Pipe;
use crate::game::{bird, pipe};
use crate::utils::request_animation_frame;
use futures::channel::oneshot;
use neat_gru::neural_network::nn::NeuralNetwork;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::sync::Arc;
use wasm_bindgen::__rt::std::sync::Mutex;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

const SPACEBAR: u32 = 32;

pub trait Render {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d);
}

/// Get the next pipe
macro_rules! get_first_pipe {
    ($self: expr) => {
        if $self.pipes[0].x + pipe::WIDTH >= bird::X - bird::RADIUS {
            &$self.pipes[0]
        } else {
            &$self.pipes[1]
        }
    };
}
struct PlayerHandler<const GAME_TYPE: i32> {
    bird: Bird<{ GAME_TYPE }>,
    space_pressed: Arc<Mutex<bool>>,
    func_keydown: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
    func_keyup: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
    func_mousedown: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
    func_mouseup: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}
/// Macro used for handling inputs
macro_rules! handle_input {
    ($space_pressed_clone: expr, $key: expr, $bool: expr) => {
        Closure::wrap(Box::new(move |js_event: web_sys::KeyboardEvent| {
            if js_event.key_code() == $key {
                *$space_pressed_clone.lock().unwrap() = $bool;
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>)
    };
    ($space_pressed_clone: expr, $bool: expr) => {
        Closure::wrap(Box::new(move |_js_event: web_sys::KeyboardEvent| {
            *$space_pressed_clone.lock().unwrap() = $bool;
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>)
    };
}

impl<const GAME_TYPE: i32> PlayerHandler<{ GAME_TYPE }> {
    pub fn new() -> PlayerHandler<{ GAME_TYPE }> {
        let space_pressed = Arc::new(Mutex::new(false));
        let pressed_clone = space_pressed.clone();
        let pressed_clone2 = space_pressed.clone();
        let pressed_clone3 = space_pressed.clone();
        let pressed_clone4 = space_pressed.clone();

        // Key Handling
        let func_keydown = handle_input!(pressed_clone, SPACEBAR, true);

        let func_keyup = handle_input!(pressed_clone2, SPACEBAR, false);

        let func_mousedown = handle_input!(pressed_clone3, true);

        let func_mouseup = handle_input!(pressed_clone4, false);

        let document = web_sys::window().unwrap().document().unwrap();

        document
            .add_event_listener_with_callback("keydown", func_keydown.as_ref().unchecked_ref())
            .unwrap();

        document
            .add_event_listener_with_callback("keyup", func_keyup.as_ref().unchecked_ref())
            .unwrap();

        document
            .add_event_listener_with_callback("mousedown", func_mousedown.as_ref().unchecked_ref())
            .unwrap();

        document
            .add_event_listener_with_callback("mouseup", func_mouseup.as_ref().unchecked_ref())
            .unwrap();

        PlayerHandler {
            space_pressed,
            func_keydown,
            func_keyup,
            func_mousedown,
            func_mouseup,
            bird: Bird::new_without_handler(usize::MAX, String::from("black")),
        }
    }

    pub fn is_pressed(&self) -> bool {
        let pressed = &mut *self.space_pressed.lock().unwrap();
        let pressed_cp = *pressed;
        if GAME_TYPE == 0 {
            *pressed = false;
        }
        pressed_cp
    }
}

impl<const GAME_TYPE: i32> Drop for PlayerHandler<{ GAME_TYPE }> {
    fn drop(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();
        document
            .remove_event_listener_with_callback(
                "keydown",
                self.func_keydown.as_ref().unchecked_ref(),
            )
            .unwrap();
        document
            .remove_event_listener_with_callback("keyup", self.func_keyup.as_ref().unchecked_ref())
            .unwrap();
        document
            .remove_event_listener_with_callback(
                "mousedown",
                self.func_mousedown.as_ref().unchecked_ref(),
            )
            .unwrap();
        document
            .remove_event_listener_with_callback(
                "mouseup",
                self.func_mouseup.as_ref().unchecked_ref(),
            )
            .unwrap();
    }
}

pub struct Game<const GAME_TYPE: i32> {
    pipes: Vec<Pipe>,
    birds: Vec<Bird<{ GAME_TYPE }>>,
    pub scores: Vec<f64>,
    rng: ThreadRng,
    width: f64,
    height: f64,
    render_count: i32,
    species_count: usize,
    generation: usize,
    hole_size: f64,
    ticks: usize,
    current_score: f64,
    pub started: bool,
    player: Option<PlayerHandler<{ GAME_TYPE }>>,
    canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
    speed: bool,
}

unsafe impl<const GAME_TYPE: i32> Send for Game<{ GAME_TYPE }> {}

unsafe impl<const GAME_TYPE: i32> Sync for Game<{ GAME_TYPE }> {}

impl<const GAME_TYPE: i32> Game<{ GAME_TYPE }> {
    pub fn new(
        width: f64,
        height: f64,
        render_count: i32,
        species_count: usize,
        generation: usize,
        hole_size: i32,
        player: bool,
        canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
        speed: bool,
    ) -> Game<{ GAME_TYPE }> {
        let hole_size = hole_size as f64;
        let (space_pressed, started) = if player {
            (Some(PlayerHandler::new()), false)
        } else {
            (None, true)
        };

        let rng = rand::thread_rng();
        Game {
            width,
            height,
            render_count,
            species_count,
            rng,
            canvas_ctx,
            generation,
            hole_size,
            player: space_pressed,
            started,
            pipes: Vec::new(),
            birds: Vec::new(),
            scores: Vec::new(),
            current_score: 0.0,
            ticks: 0,
            speed,
        }
    }

    pub async fn run_game(
        width: f64,
        height: f64,
        render_count: i32,
        species_count: usize,
        generation: usize,
        hole_size: i32,
        networks: Vec<NeuralNetwork<f64>>,
    ) -> Arc<Mutex<Game<{ GAME_TYPE }>>> {
        let game = {
            let document = web_sys::window().unwrap().document().unwrap();
            // Player Checkbox
            let player_checkbox = document.get_element_by_id("player").unwrap();
            let player_checkbox = player_checkbox
                .dyn_into::<web_sys::HtmlInputElement>()
                .map_err(|_| ())
                .unwrap();
            let player_checked = player_checkbox.checked();

            // Speed Checkbox
            let speed_checkbox = document.get_element_by_id("speed").unwrap();
            let speed_checkbox = speed_checkbox.dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| ())
            .unwrap();
            let speed_check = speed_checkbox.checked();
            let canvas = document.get_element_by_id("canvas").unwrap();

            let canvas: Arc<web_sys::HtmlCanvasElement> = Arc::new(
                canvas
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .map_err(|_| ())
                    .unwrap(),
            );
            let context = Arc::new(Mutex::new(
                canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap(),
            ));
            Arc::new(Mutex::new(Game::<GAME_TYPE>::new(
                width,
                height,
                render_count,
                species_count,
                generation,
                hole_size,
                player_checked,
                context,
                speed_check,
            )))
        };
        let game_cp = game.clone();

        let (sender, receiver) = oneshot::channel::<()>();

        {
            let sender = Arc::new(Mutex::new(Some(sender)));
            {
                game.lock().unwrap().init(networks);
            }
            let f = Arc::new(Mutex::new(None));
            let g = f.clone();

            *g.lock().unwrap() = Some(Closure::wrap(Box::new(move || {
                let game_obj = &mut *game.lock().unwrap();
                if !game_obj.started && !game_obj.check_started() {
                    game_obj.render_waiting();
                    request_animation_frame(f.lock().unwrap().as_ref().unwrap());
                    return;
                }
                game_obj.make_decisions();
                game_obj.game_logic();
                game_obj.handle_collisions();
                game_obj.render();
                if !game_obj.ended() {
                    request_animation_frame(f.lock().unwrap().as_ref().unwrap());
                } else {
                    let lock = sender.lock().unwrap().take();
                    lock.unwrap().send(()).unwrap();
                }
            }) as Box<dyn FnMut()>));
            {
                request_animation_frame(g.lock().unwrap().as_ref().unwrap());
            }
        }
        receiver.await.unwrap();
        game_cp
    }

    fn check_started(&mut self) -> bool {
        let started = self.player.as_ref().unwrap().is_pressed();
        if started {
            self.started = true;
        }
        started
    }

    fn add_pipe(&mut self) {
        let y = self.height
            - self
                .rng
                .gen_range(self.height * 0.0..(self.height - self.hole_size));

        match self.pipes.last() {
            None => {
                self.pipes.push(Pipe::new(
                    self.width,
                    0.25 * self.height + 0.5 * y,
                    self.hole_size,
                ));
            }
            Some(Pipe { x, .. }) => {
                let x = *x;
                self.pipes.push(Pipe::new(x + 500.0, y, self.hole_size));
            }
        }
    }

    fn get_speed(&self) -> f64{
        let mut result = 4.0;
        result += self.get_speed_increase();
        result
    }

    /// Returns the speed increase. 0.0 if speed increase is not activated
    fn get_speed_increase(&self) -> f64{
        let mut result = 0.0;
        if self.speed{
            result+=((self.ticks as f64) * 0.002).tanh() * 2.5;
        }
        result
    }

    /// Moves the pipes to the left
    fn move_pipes(&mut self) {
        let speed = self.get_speed();
        for pipe in &mut self.pipes {
            pipe.move_left(speed);
        }
        if self.pipes[0].x <= -pipe::WIDTH {
            self.pipes.remove(0);
            self.add_pipe();
            self.current_score += 1.0;
        }
    }

    /// Applies "gravity" to every bird
    fn apply_birds_velocity(&mut self) {
        for bird in &mut self.birds {
            bird.y_velocity();
        }
        if let Some(player) = &mut self.player {
            player.bird.y_velocity();
        }
    }

    pub fn make_decisions(&mut self) {
        let first_pipe = get_first_pipe!(self);

        let mut inputs = [(first_pipe.x * 2.0 - self.width) / self.width, 0., 0.];

        for bird in &mut self.birds {
            inputs[1] = (bird.y - first_pipe.hole) / self.height;
            inputs[2] = 0.01 * bird.velocity;
            bird.make_decision(&inputs);
        }
        if let Some(player) = &mut self.player {
            if player.is_pressed() {
                player.bird.jump();
            }
        }
    }

    fn handle_pipe_collision(&mut self, index: usize) {
        let pipe_ref = &self.pipes[index];
        let overlap_x = pipe_ref.x <= bird::X + bird::RADIUS
            && pipe_ref.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        let current_score = self.ticks as f64;

        let scores = &mut self.scores;
        let hole_size = self.hole_size;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                let alive = !(bird_ref.y + bird::RADIUS >= pipe_ref.y
                    || bird_ref.y - bird::RADIUS <= pipe_ref.y - hole_size);
                if !alive {
                    scores[bird_ref.index] = current_score;
                }
                alive
            });

            if let Some(player) = &mut self.player {
                let player_bird = &player.bird;
                let alive = !(player_bird.y + bird::RADIUS >= pipe_ref.y
                    || player_bird.y - bird::RADIUS <= pipe_ref.y - hole_size);
                if !alive {
                    self.player.take();
                }
            }
        }
    }

    pub fn handle_collisions(&mut self) {
        let height = self.height;
        let current_score = self.ticks as f64;
        self.ticks += 1;
        let scores = &mut self.scores;

        self.birds.retain(|bird_ref| {
            let alive = bird_ref.y + bird::RADIUS <= height && bird_ref.y - bird::RADIUS >= 0.0;
            if !alive {
                scores[bird_ref.index] = current_score;
            }
            alive
        });

        if let Some(player) = &mut self.player {
            let player_bird = &player.bird;
            let alive =
                player_bird.y + bird::RADIUS <= height && player_bird.y - bird::RADIUS >= 0.0;
            if !alive {
                self.player.take();
            }
        }

        self.handle_pipe_collision(0);
        self.handle_pipe_collision(1);
    }

    pub fn game_logic(&mut self) {
        self.move_pipes();
        self.apply_birds_velocity();
    }

    fn random_color(&mut self) -> String {
        // High value to not mix up with player
        let c1 = self.rng.gen_range(100..255);
        let c2 = self.rng.gen_range(100..255);
        let c3 = self.rng.gen_range(100..255);

        format!("rgb({}, {}, {})", c1, c2, c3)
    }

    pub fn init(&mut self, nets: Vec<NeuralNetwork<f64>>) {
        for _ in 0..5 {
            self.add_pipe();
        }
        for (index, net) in nets.into_iter().enumerate() {
            let random_color = self.random_color();
            self.birds.push(Bird::new(index, random_color, net));
            self.scores.push(0.0);
        }
    }

    pub fn render_waiting(&self) {
        let canvas_ctx = &*self.canvas_ctx.lock().unwrap();
        canvas_ctx.set_fill_style(&JsValue::from_str("rgba(50, 50, 50, 0.01)"));
        canvas_ctx.rect(0.0, 0.0, self.width, self.height);
        canvas_ctx.fill();
        canvas_ctx.set_font("20px Arial");
        canvas_ctx.set_fill_style(&JsValue::from_str("white"));
        canvas_ctx
            .fill_text(
                "Press space or click to play",
                self.width / 2.0 - 140.0,
                self.height / 2.0 + 15.0,
            )
            .unwrap();
    }

    pub fn render(&self) {
        let canvas_ctx = &*self.canvas_ctx.lock().unwrap();
        canvas_ctx.clear_rect(0.0, 0.0, self.width, self.height);
        for bird in self.birds.iter().take(self.render_count as usize) {
            bird.render(canvas_ctx);
        }
        if let Some(player) = &self.player {
            let player_bird = &player.bird;
            player_bird.render(canvas_ctx);
        }
        for pipe in &self.pipes {
            pipe.render(canvas_ctx);
        }
        canvas_ctx.set_font("30px Arial");
        canvas_ctx.set_fill_style(&JsValue::from_str("black"));
        canvas_ctx
            .fill_text(
                &*format!("{}", self.current_score),
                self.width / 2.0 - 30.0,
                30.0,
            )
            .unwrap();
        canvas_ctx
            .fill_text(
                &*format!("Alive: {}", self.birds.len()),
                self.width / 2.0 - 45.0,
                self.height - 90.0,
            )
            .unwrap();
        canvas_ctx
            .fill_text(
                &*format!("Species: {}", self.species_count),
                self.width / 2.0 - 75.0,
                self.height - 60.0,
            )
            .unwrap();
        canvas_ctx
            .fill_text(
                &*format!("Generation: {}", self.generation),
                self.width / 2.0 - 90.0,
                self.height - 30.0,
            )
            .unwrap();
    }

    pub fn ended(&self) -> bool {
        self.birds.is_empty() && self.player.is_none()
    }
}
