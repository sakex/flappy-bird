use crate::game::game::Render;
use wasm_bindgen::JsValue;

pub const RADIUS: f64 = 30.0;
pub const X: f64 = 45.0;

pub struct Bird {
    pub index: usize,
    pub y: f64,
    velocity: f64,
}

impl Bird {
    pub fn new(index: usize) -> Bird {
        Bird {
            index,
            y: 400.0,
            velocity: 0.0
        }
    }

    pub fn y_velocity(&mut self) {
        self.y -= self.velocity;
        self.velocity -= 0.5;
    }

    pub fn jump(&mut self) {
        self.velocity = 10.0;
    }
}

impl Render for Bird {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        canvas_ctx.begin_path();
        canvas_ctx.set_fill_style(&JsValue::from_str("#fcf025"));
        if let Err(_) = canvas_ctx
            .arc(X, self.y, RADIUS, 0.0, std::f64::consts::PI * 2.0) {
            crate::log("An error happened rendering bird");
        }
        canvas_ctx.fill();
    }
}