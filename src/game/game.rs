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
        canvas_ctx: Arc<Mutex<web_sys::CanvasRenderingContext2d>>,
    ) -> Game {
        let rng = rand::thread_rng();
        Game {
            width,
            height,
            species_count,
            rng,
            canvas_ctx,
            generation,
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
            Arc::new(Mutex::new(Game::new(width, height, species_count, generation, context)))
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
        let y = self.height - self.rng.gen_range(self.height * 0.1..(self.height * 0.9 - HOLE_SIZE));

        match self.pipes.last() {
            None => {
                self.pipes.push(Pipe::new(self.width, 0.25 * self.height + 0.5 * y));
            }
            Some(Pipe { x, .. }) => {
                let x = *x;
                self.pipes.push(Pipe::new(x + 500.0, y));
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
        let first_pipe = if self.pipes[0].x + pipe::WIDTH >= bird::X - bird::RADIUS {
            &self.pipes[0]
        } else {
            &self.pipes[1]
        };

        let mut inputs = [
            (first_pipe.x * 2.0 - self.width) / self.width,
            0.,
            0.
        ];

        for bird in &mut self.birds {
            inputs[1] = (bird.y - first_pipe.hole) / self.height;
            inputs[2] = 0.01 * bird.velocity;
            bird.make_decision(&inputs);
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

        let first_pipe = &self.pipes[0];
        let overlap_x = first_pipe.x <= bird::X + bird::RADIUS
            && first_pipe.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                let alive = !(bird_ref.y + bird::RADIUS >= first_pipe.y || bird_ref.y - bird::RADIUS <= first_pipe.y - HOLE_SIZE);
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
                let alive = !(bird_ref.y + bird::RADIUS >= second_pipe.y || bird_ref.y - bird::RADIUS <= second_pipe.y - HOLE_SIZE);
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
        for bird in self.birds.iter().take(250) {
            bird.render(&canvas_ctx);
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
        self.birds.is_empty()
    }
}
