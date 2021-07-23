use crate::game::game::Render;
use wasm_bindgen::JsValue;

pub const WIDTH: f64 = 100.0;
pub const BORDER_WIDTH: f64 = 20.0;
pub const LINE_WIDTH: f64 = 5.0;

pub struct Pipe {
    pub x: f64,
    pub y: f64,
    pub hole: f64,
    hole_size: f64,
}

impl Pipe {
    pub fn new(x: f64, y: f64, hole_size: f64) -> Pipe {
        Pipe {
            x,
            y,
            hole_size,
            hole: y - hole_size / 2.,
        }
    }

    pub fn move_left(&mut self, speed: f64) {
        self.x -= 1.0 * speed;
    }
}

impl Render for Pipe {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        let h_size = self.hole_size;
        let height = 800.0 - h_size;
        canvas_ctx.begin_path();
        canvas_ctx.set_fill_style(&JsValue::from_str("#6ebb2d"));
        canvas_ctx.set_stroke_style(&JsValue::from_str("black"));
        canvas_ctx.set_line_width(LINE_WIDTH);
        canvas_ctx.rect(self.x, self.y + BORDER_WIDTH, WIDTH, height);
        canvas_ctx.fill();
        canvas_ctx.rect(self.x, self.y + BORDER_WIDTH, WIDTH, height);
        canvas_ctx.stroke();
        canvas_ctx.rect(self.x - 5.0, self.y, WIDTH + 10.0, BORDER_WIDTH);
        canvas_ctx.fill();
        canvas_ctx.rect(self.x - 5.0, self.y, WIDTH + 10.0, BORDER_WIDTH);
        canvas_ctx.stroke();

        canvas_ctx.rect(self.x, self.y - h_size - BORDER_WIDTH, WIDTH, -height);
        canvas_ctx.fill();
        canvas_ctx.rect(self.x, self.y - h_size - BORDER_WIDTH, WIDTH, -height);
        canvas_ctx.stroke();
        canvas_ctx.rect(self.x - 5.0, self.y - h_size, WIDTH + 10.0, -BORDER_WIDTH);
        canvas_ctx.fill();
        canvas_ctx.rect(self.x - 5.0, self.y - h_size, WIDTH + 10.0, -BORDER_WIDTH);
        canvas_ctx.stroke();
    }
}
