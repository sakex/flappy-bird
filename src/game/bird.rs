use crate::game::game::Render;
use neat_gru::neural_network::nn::NeuralNetwork;
use wasm_bindgen::JsValue;

pub const RADIUS: f64 = 30.0;
pub const X: f64 = 45.0;

pub struct Bird {
    pub index: usize,
    pub y: f64,
    color: String,
    pub velocity: f64,
    net: NeuralNetwork<f64>,
}

impl Bird {
    pub fn new(index: usize, color: String, net: NeuralNetwork<f64>) -> Bird {
        Bird {
            index,
            net,
            color,
            y: 400.0,
            velocity: 0.0,
        }
    }

    pub fn y_velocity(&mut self) {
        self.y -= self.velocity;
        self.velocity -= 0.5;
    }

    fn jump(&mut self) {
        self.velocity += 1.0;
    }

    pub fn make_decision(&mut self, inputs: &[f64]) {
        let output = self.net.compute(inputs);
        if output[0] >= output[1] {
            self.jump();
        }
    }
}

impl Render for Bird {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        let black = JsValue::from_str("black");
        canvas_ctx.begin_path();
        canvas_ctx.set_stroke_style(&black);
        canvas_ctx.set_line_width(5.0);
        canvas_ctx.set_fill_style(&JsValue::from_str(&*self.color));
        canvas_ctx.arc(X, self.y, RADIUS, 0.0, std::f64::consts::PI * 2.0).unwrap();
        canvas_ctx.fill();
        // Eye
        canvas_ctx.begin_path();
        canvas_ctx.set_line_width(5.0);
        canvas_ctx.set_fill_style(&JsValue::from_str("white"));
        canvas_ctx.arc(X + RADIUS / 3.0, self.y - RADIUS / 2.0, RADIUS / 2.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
        canvas_ctx.fill();
        // Eye dot
        canvas_ctx.begin_path();
        canvas_ctx.arc(X + RADIUS / 2.0, self.y - RADIUS / 2.0, 5.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
        canvas_ctx.set_fill_style(&black);
        canvas_ctx.fill();

        // Mouth
        canvas_ctx.begin_path();
        canvas_ctx.set_fill_style(&JsValue::from_str("#f76946"));
        canvas_ctx.ellipse(X + RADIUS / 1.2, self.y + 5.0, RADIUS / 2.0, RADIUS / 2.8, 0.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
        canvas_ctx.fill();
        // Wing
        canvas_ctx.begin_path();
        canvas_ctx.set_fill_style(&JsValue::from_str("#f7ea25"));
        canvas_ctx.set_stroke_style(&black);
        canvas_ctx.ellipse(X - RADIUS / 1.5, self.y + RADIUS / 2.0, RADIUS / 2.0, RADIUS / 3.0, std::f64::consts::PI * 1.9, 0.0, std::f64::consts::PI * 2.0).unwrap();
        canvas_ctx.fill();
    }
}
