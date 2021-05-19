use crate::game::game::Render;
use wasm_bindgen::JsValue;

pub const WIDTH: f64 = 100.0;
pub const BORDER_WIDTH: f64 = 20.0;
pub const LINE_WIDTH: f64 = 5.0;

pub const fn height(game_type: i32) -> f64 {
    if game_type == 0 {
        500.0
    } else {
        750.0
    }
}

pub const fn hole_size(game_type: i32) -> f64 {
    if game_type == 1 {
        80.0
    } else {
        200.0
    }
}

pub struct Pipe<const GAME_TYPE: i32> {
    pub x: f64,
    pub y: f64,
    pub hole: f64,
}

impl<const GAME_TYPE: i32> Pipe<{ GAME_TYPE }> {
    pub fn new(x: f64, y: f64) -> Pipe<{ GAME_TYPE }> {
        Pipe {
            x,
            y,
            hole: y - hole_size(GAME_TYPE) / 2.,
        }
    }

    pub fn move_left(&mut self) {
        self.x -= 5.0;
    }
}

impl<const GAME_TYPE: i32> Render for Pipe<{ GAME_TYPE }> {
    fn render(&self, canvas_ctx: &web_sys::CanvasRenderingContext2d) {
        let h_size = hole_size(GAME_TYPE);
        let height = height(GAME_TYPE);
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
        canvas_ctx.rect(
            self.x - 5.0,
            self.y - h_size,
            WIDTH + 10.0,
            -BORDER_WIDTH,
        );
        canvas_ctx.fill();
        canvas_ctx.rect(
            self.x - 5.0,
            self.y - h_size,
            WIDTH + 10.0,
            -BORDER_WIDTH,
        );
        canvas_ctx.stroke();
    }
}
