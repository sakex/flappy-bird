use crate::game::game::Render;
use neat_gru::neural_network::nn::NeuralNetwork;
use wasm_bindgen::JsValue;

pub const RADIUS: f64 = 30.0;
pub const X: f64 = 45.0;

pub struct Bird<const GAME_TYPE: i32> {
    pub index: usize,
    pub y: f64,
    color: String,
    pub velocity: f64,
    net: Option<NeuralNetwork<f64>>,
}

impl<const GAME_TYPE: i32> Bird<{ GAME_TYPE }> {
    pub fn new(index: usize, color: String, net: NeuralNetwork<f64>) -> Bird<{ GAME_TYPE }> {
        Bird {
            index,
            color,
            net: Some(net),
            y: 400.0,
            velocity: 0.0,
        }
    }

    pub fn new_without_handler(index: usize, color: String) -> Bird<{ GAME_TYPE }> {
        Bird {
            index,
            color,
            net: None,
            y: 400.0,
            velocity: 0.0,
        }
    }

    pub fn y_velocity(&mut self) {
        self.y -= self.velocity;
        self.velocity -= 0.5;
    }

    pub fn jump(&mut self) {
        if GAME_TYPE == 0 {
            self.velocity = 10.0;
        } else if GAME_TYPE == 1 {
            self.velocity += 1.0;
        }
    }

    /// Computes the output of the agent which then takes an action
    pub fn make_decision(&mut self, inputs: &[f64]) {
        let output = self.net.as_mut().unwrap().compute(inputs);
        // We can use the very useful Rust Pattern matching here
        match GAME_TYPE{
            0 if output[0]>=0.0 => self.jump(),
            1 if output[0]>=0.0 => self.jump(),
            _ => panic!("Invalid Game type")
        }
    }
}

impl<const GAME_TYPE: i32> Render for Bird<{ GAME_TYPE }> {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        let black = JsValue::from_str("black");
        let is_player = self.net.is_none();
        canvas_ctx.begin_path();
        canvas_ctx.set_line_width(5.0);
        canvas_ctx.set_fill_style(&JsValue::from_str(&*self.color));
        canvas_ctx
            .arc(X, self.y, RADIUS, 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        canvas_ctx.fill();
        // Eye
        canvas_ctx.begin_path();
        canvas_ctx.set_line_width(5.0);
        canvas_ctx.set_fill_style(&JsValue::from_str("white"));
        canvas_ctx
            .arc(
                X + RADIUS / 3.0,
                self.y - RADIUS / 2.0,
                RADIUS / 2.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        canvas_ctx.fill();

        // Eye dot
        canvas_ctx.begin_path();
        canvas_ctx
            .arc(
                X + RADIUS / 2.0,
                self.y - RADIUS / 2.0,
                5.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        canvas_ctx.set_fill_style(&black);
        canvas_ctx.fill();
        // Mouth
        canvas_ctx.begin_path();
        if !is_player {
            canvas_ctx.set_fill_style(&JsValue::from_str("#f76946"));
        }
        canvas_ctx
            .ellipse(
                X + RADIUS / 1.2,
                self.y + 5.0,
                RADIUS / 2.0,
                RADIUS / 2.8,
                0.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        canvas_ctx.fill();
        // Wing
        canvas_ctx.begin_path();
        if !is_player {
            canvas_ctx.set_fill_style(&JsValue::from_str("#f7ea25"));
        }
        canvas_ctx
            .ellipse(
                X - RADIUS / 1.5,
                self.y + RADIUS / 2.0,
                RADIUS / 2.0,
                RADIUS / 3.0,
                std::f64::consts::PI * 1.9,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
        canvas_ctx.fill();
    }
}
