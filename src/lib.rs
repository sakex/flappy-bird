extern crate serde;

mod utils;
mod game;

use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use game::game::Game;
use std::rc::Rc;
use std::cell::RefCell;
use crate::utils::request_animation_frame;
use serde::{Serialize, Deserialize};
use web_sys::window;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
pub struct JsEvent {
    pub which: i32,
}

#[wasm_bindgen]
pub struct ClosuresHandle {
    _keypress: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

#[wasm_bindgen]
pub fn start() -> ClosuresHandle {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let space_pressed = Rc::new(RefCell::new(false));
    let pressed_clone = space_pressed.clone();


    let func = Closure::wrap(Box::new(move |js_event: web_sys::KeyboardEvent| {
        *pressed_clone.borrow_mut() = js_event.key_code() == 32;
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);


    document.add_event_listener_with_callback("keydown", func.as_ref().unchecked_ref());

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let game = Rc::new(RefCell::new(Game::new(700.0, 800.0, context)));
    game.borrow_mut().init();
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let started = Rc::new(RefCell::new(false));

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let started_ref = &mut *started.borrow_mut();
        let game_obj = &mut *game.borrow_mut();
        {
            let pressed = &mut *space_pressed.borrow_mut();
            if *pressed {
                if *started_ref {
                    game_obj.jump();
                }
                else {
                    *started_ref = true;
                }
            }
            *pressed = false;
        }
        if *started_ref {
            game_obj.game_logic();
            game_obj.handle_collisions();
        }
        game_obj.render();
        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());


    ClosuresHandle {
        _keypress: func
    }
}