use crate::game::bird::Bird;
use crate::game::pipe::Pipe;
use crate::game::{bird, pipe};
use crate::utils::request_animation_frame;
use futures::channel::oneshot;
use neat_gru::neural_network::nn::NeuralNetwork;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::__rt::std::sync::Mutex;

pub trait Render {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d);
}

pub struct Game {
    pipes: Vec<Pipe>,
    birds: Vec<Bird>,
    pub scores: Vec<f64>,
    rng: ThreadRng,
    last_pipe_up: bool,
    width: f64,
    height: f64,
    current_score: f64,
    canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
}

unsafe impl Send for Game {}

unsafe impl Sync for Game {}

impl Game {
    pub fn new(
        width: f64,
        height: f64,
        canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
    ) -> Game {
        let rng = rand::thread_rng();
        Game {
            width,
            height,
            rng,
            canvas_ctx,
            pipes: Vec::new(),
            birds: Vec::new(),
            scores: Vec::new(),
            current_score: 0.0,
            last_pipe_up: false,
        }
    }

    pub async fn run_game(
        width: f64,
        height: f64,
        networks: Vec<NeuralNetwork<f64>>,
    ) -> Arc<Mutex<Game>> {
        let game = {
            let document = web_sys::window().unwrap().document().unwrap();
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
            Arc::new(Mutex::new(Game::new(width, height, context)))
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

    fn add_pipe(&mut self) {
        let y = if self.last_pipe_up {
            -self.rng.gen_range(0.0..(pipe::HEIGHT / 4.0))
        } else {
            self.height - self.rng.gen_range(0.0..(pipe::HEIGHT / 2.0)) - pipe::HEIGHT / 2.0
        };
        self.last_pipe_up = !self.last_pipe_up;

        match self.pipes.last() {
            None => {
                self.pipes.push(Pipe::new(bird::X + 30.0, y));
            }
            Some(Pipe { x, .. }) => {
                let x = *x;
                self.pipes.push(Pipe::new(x + pipe::WIDTH * 2.0, y));
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
    }

    pub fn make_decisions(&mut self) {
        let first_pipe = &self.pipes[0];
        let second_pipe = &self.pipes[1];

        let first_x_input = (first_pipe.x * 2.0 - self.width) / self.width;
        let first_y_input = (first_pipe.y * 2.0 - self.height) / self.height;

        let second_x_input = (second_pipe.x * 2.0 - self.width) / self.width;
        let second_y_input = (second_pipe.y * 2.0 - self.height) / self.height;

        let mut inputs = vec![
            first_x_input,
            first_y_input,
            second_x_input,
            second_y_input,
            0.0,
        ];

        for bird in &mut self.birds {
            inputs[4] = (bird.y * 2.0 - self.height) / self.height;
            bird.make_decision(&inputs);
        }
    }

    pub fn handle_collisions(&mut self) {
        let height = self.height;
        let current_score = self.current_score;
        let scores = &mut self.scores;
        self.birds.retain(|bird_ref| {
            let alive = bird_ref.y + bird::RADIUS <= height && bird_ref.y - bird::RADIUS >= 0.0;
            if !alive {
                scores[bird_ref.index] = current_score;
            }
            alive
        });

        let first_pipe = &self.pipes[0];
        let overlap_x = first_pipe.x <= bird::X + bird::RADIUS
            && first_pipe.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                let alive = !(bird_ref.y + bird::RADIUS >= first_pipe.y
                    && bird_ref.y - bird::RADIUS <= first_pipe.y + pipe::HEIGHT);
                if !alive {
                    scores[bird_ref.index] = current_score;
                }
                alive
            });
        }
        let second_pipe = &self.pipes[1];

        let overlap_x = second_pipe.x <= bird::X + bird::RADIUS
            && second_pipe.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                let alive = bird_ref.y + bird::RADIUS <= second_pipe.y
                    && bird_ref.y >= second_pipe.y + pipe::HEIGHT;
                if !alive {
                    scores[bird_ref.index] = current_score;
                }
                alive
            });
        }
    }

    pub fn game_logic(&mut self) {
        self.move_pipes();
        self.apply_birds_velocity();
    }

    fn random_color(&mut self) -> String {
        let c1 = self.rng.gen_range(0..255);
        let c2 = self.rng.gen_range(0..255);
        let c3 = self.rng.gen_range(0..255);

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
        for bird in &self.birds {
            bird.render(&canvas_ctx);
        }
        for pipe in &self.pipes {
            pipe.render(&canvas_ctx);
        }
    }

    pub fn ended(&self) -> bool {
        self.birds.is_empty()
    }
}
