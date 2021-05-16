use crate::game::game::Render;
use wasm_bindgen::JsValue;

pub const WIDTH: f64 = 100.0;
pub const HEIGHT: f64 = 320.0;

pub struct Pipe {
    pub x: f64,
    pub y: f64,
}

impl Pipe {
    pub fn new(x: f64, y: f64) -> Pipe {
        Pipe { x, y }
    }

    pub fn move_left(&mut self) {
        self.x -= 5.0;
    }
}

impl Render for Pipe {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        canvas_ctx.begin_path();
        canvas_ctx.set_fill_style(&JsValue::from_str("#6ebb2d"));
        canvas_ctx.set_stroke_style(&JsValue::from_str("black"));
        canvas_ctx.set_line_width(5.0);
        canvas_ctx.rect(self.x, self.y, WIDTH, HEIGHT);
        canvas_ctx.fill();
        canvas_ctx.rect(self.x, self.y, WIDTH, HEIGHT);
        canvas_ctx.stroke();
    }
}
