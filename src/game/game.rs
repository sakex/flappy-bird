use crate::game::pipe::Pipe;
use crate::game::bird::Bird;
use rand::Rng;
use rand::prelude::ThreadRng;
use crate::game::{pipe, bird};

pub trait Render {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d);
}

pub struct Game {
    pipes: Vec<Pipe>,
    birds: Vec<Bird>,
    scores: Vec<f64>,
    rng: ThreadRng,
    last_pipe_up: bool,
    width: f64,
    height: f64,
    current_score: f64,
    canvas_ctx: web_sys::CanvasRenderingContext2d,
}

impl Game {
    pub fn new(width: f64, height: f64, canvas_ctx: web_sys::CanvasRenderingContext2d) -> Game {
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

    pub fn jump(&mut self) {
        self.birds[0].jump();
    }

    pub fn handle_collisions(&mut self) {
        let height = self.height;
        self.birds.retain(|bird_ref| {
            bird_ref.y + bird::RADIUS <= height && bird_ref.y - bird::RADIUS >= 0.0
        });

        let first_pipe = &self.pipes[0];
        let overlap_x = first_pipe.x <= bird::X + bird::RADIUS && first_pipe.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                !(bird_ref.y + bird::RADIUS >= first_pipe.y && bird_ref.y - bird::RADIUS <= first_pipe.y + pipe::HEIGHT)
            });
        }
        let second_pipe = &self.pipes[1];

        let overlap_x = second_pipe.x <= bird::X + bird::RADIUS && second_pipe.x + pipe::WIDTH >= bird::X - bird::RADIUS;
        if overlap_x {
            self.birds.retain(|bird_ref| {
                bird_ref.y + bird::RADIUS <= second_pipe.y && bird_ref.y >= second_pipe.y + pipe::HEIGHT
            });
        }
    }

    pub fn game_logic(&mut self) {
        self.move_pipes();
        self.apply_birds_velocity();
    }

    pub fn init(&mut self) {
        for _ in 0..5 {
            self.add_pipe();
        }
        for i in 0..10 {
            self.birds.push(Bird::new(i));
            self.scores.push(0.0);
        }
    }

    pub fn render(&self) {
        self.canvas_ctx.clear_rect(0.0, 0.0, self.width, self.height);
        for bird in &self.birds {
            bird.render(&self.canvas_ctx);
        }
        for pipe in &self.pipes {
            pipe.render(&self.canvas_ctx);
        }
    }
}