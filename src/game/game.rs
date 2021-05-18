use crate::game::bird::Bird;
use crate::game::pipe::{Pipe, HOLE_SIZE};
use crate::game::{bird, pipe};
use crate::utils::request_animation_frame;
use futures::channel::oneshot;
use neat_gru::neural_network::nn::NeuralNetwork;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::__rt::std::sync::Mutex;

pub trait Render {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d);
}

struct PlayerHandler {
    bird: Bird,
    space_pressed: Arc<Mutex<bool>>,
    event_listener: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

impl PlayerHandler {
    pub fn new() -> PlayerHandler {
        let space_pressed = Arc::new(Mutex::new(false));
        let pressed_clone = space_pressed.clone();

        let func = Closure::wrap(Box::new(move |js_event: web_sys::KeyboardEvent| {
            *pressed_clone.lock().unwrap() = js_event.key_code() == 32;
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        let document = web_sys::window().unwrap().document().unwrap();

        document
            .add_event_listener_with_callback("keydown", func.as_ref().unchecked_ref())
            .unwrap();

        PlayerHandler {
            space_pressed,
            event_listener: func,
            bird: Bird::new_without_handler(usize::MAX, String::from("black")),
        }
    }

    pub fn is_pressed(&self) -> bool {
        let pressed = &mut *self.space_pressed.lock().unwrap();
        let pressed_cp = *pressed;
        *pressed = false;
        pressed_cp
    }
}

impl Drop for PlayerHandler {
    fn drop(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();
        document.remove_event_listener_with_callback("keydown", &self.event_listener.as_ref().unchecked_ref()).unwrap();
    }
}


pub struct Game {
    pipes: Vec<Pipe>,
    birds: Vec<Bird>,
    pub scores: Vec<f64>,
    rng: ThreadRng,
    width: f64,
    height: f64,
    species_count: usize,
    generation: usize,
    ticks: usize,
    current_score: f64,
    pub started: bool,
    player: Option<PlayerHandler>,
    canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
}

unsafe impl Send for Game {}

unsafe impl Sync for Game {}

impl Game {
    pub fn new(
        width: f64,
        height: f64,
        species_count: usize,
        generation: usize,
        player: bool,
        canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
    ) -> Game {
        let (space_pressed, started) = if player {
            (Some(PlayerHandler::new()), false)
        } else {
            (None, true)
        };


        let rng = rand::thread_rng();
        Game {
            width,
            height,
            species_count,
            rng,
            canvas_ctx,
            generation,
            player: space_pressed,
            started,
            pipes: Vec::new(),
            birds: Vec::new(),
            scores: Vec::new(),
            current_score: 0.0,
            ticks: 0,
        }
    }

    pub async fn run_game(
        width: f64,
        height: f64,
        species_count: usize,
        generation: usize,
        networks: Vec<NeuralNetwork<f64>>,
    ) -> Arc<Mutex<Game>> {
        let game = {
            let document = web_sys::window().unwrap().document().unwrap();
            let player_checkbox = document.get_element_by_id("player").unwrap();
            let player_checkbox = player_checkbox
                .dyn_into::<web_sys::HtmlInputElement>()
                .map_err(|_| ())
                .unwrap();
            let player_checked = player_checkbox.checked();
            let start_button = document.get_element_by_id("start").unwrap();
            start_button.set_attribute("style", "display: none;");

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
            Arc::new(Mutex::new(Game::new(width, height, species_count, generation, player_checked, context)))
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
                if !game_obj.started {
                    if !game_obj.check_started() {
                        request_animation_frame(f.lock().unwrap().as_ref().unwrap());
                        return;
                    }
                }
                game_obj.make_decisions();
                game_obj.game_logic();
                game_obj.handle_collisions();
                game_obj.render();
                if !game_obj.ended() {
                    request_animation_frame(f.lock().unwrap().as_ref().unwrap());
                } else {
                    let document = web_sys::window().unwrap().document().unwrap();
                    let start_button = document.get_element_by_id("start").unwrap();
                    start_button.set_attribute("style", "display: none;");

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
        let y = self.height - self.rng.gen_range(self.height * 0.2..(self.height * 0.8 - HOLE_SIZE));

        match self.pipes.last() {
            None => {
                self.pipes.push(Pipe::new(self.width, y));
            }
            Some(Pipe { x, .. }) => {
                let x = *x;
                self.pipes.push(Pipe::new(x + pipe::WIDTH * 5.0, y));
            }
        }
    }

    fn move_pipes(&mut self) {
        for pipe in &mut self.pipes {
            pipe.move_left();
        }
        if self.pipes[0].x <= -pipe::WIDTH {
            self.pipes.remove(0);
            self.add_pipe();
            self.current_score += 1.0;
        }
    }

    fn apply_birds_velocity(&mut self) {
        for bird in &mut self.birds {
            bird.y_velocity();
        }
        if let Some(player) = &mut self.player {
            player.bird.y_velocity();
        }
    }

    pub fn make_decisions(&mut self) {
        let first_pipe = if self.pipes[0].x + pipe::WIDTH >= bird::X - bird::RADIUS {
            &self.pipes[0]
        } else {
            &self.pipes[1]
        };

        let first_x_input = (first_pipe.x * 2.0 - self.width) / self.width;
        let first_y_input = (first_pipe.hole * 2.0 - self.height) / self.height;

        let mut inputs = [
            first_x_input,
            first_y_input,
            0.0,
        ];

        for bird in &mut self.birds {
            inputs[2] = (bird.y * 2.0 - self.height) / self.height;
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
        if overlap_x {
            self.birds.retain(|bird_ref| {
                let alive = !(bird_ref.y + bird::RADIUS >= pipe_ref.y || bird_ref.y - bird::RADIUS <= pipe_ref.y - HOLE_SIZE);
                if !alive {
                    scores[bird_ref.index] = current_score;
                }
                alive
            });

            if let Some(player) = &mut self.player {
                let player_bird = &player.bird;
                let alive = !(player_bird.y + bird::RADIUS >= pipe_ref.y || player_bird.y - bird::RADIUS <= pipe_ref.y - HOLE_SIZE);
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
            let alive = player_bird.y + bird::RADIUS <= height && player_bird.y - bird::RADIUS >= 0.0;
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

    pub fn render(&self) {
        let canvas_ctx = &*self.canvas_ctx.lock().unwrap();
        canvas_ctx.clear_rect(0.0, 0.0, self.width, self.height);
        for bird in self.birds.iter().take(250) {
            bird.render(&canvas_ctx);
        }
        if let Some(player) = &self.player {
            let player_bird = &player.bird;
            player_bird.render(&canvas_ctx);
        }
        for pipe in &self.pipes {
            pipe.render(&canvas_ctx);
        }
        canvas_ctx.set_font("30px Arial");
        canvas_ctx.set_fill_style(&JsValue::from_str("black"));
        canvas_ctx.fill_text(&*format!("{}", self.current_score), self.width / 2.0 - 30.0, 30.0).unwrap();
        canvas_ctx.fill_text(&*format!("Alive: {}", self.birds.len()), self.width / 2.0 - 45.0, self.height - 90.0).unwrap();
        canvas_ctx.fill_text(&*format!("Species: {}", self.species_count), self.width / 2.0 - 75.0, self.height - 60.0).unwrap();
        canvas_ctx.fill_text(&*format!("Generation: {}", self.generation), self.width / 2.0 - 90.0, self.height - 30.0).unwrap();
    }

    pub fn ended(&self) -> bool {
        self.birds.is_empty() && self.player.is_none()
    }
}
