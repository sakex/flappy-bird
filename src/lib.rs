extern crate serde;

mod game;
mod training_simulation;
mod utils;

use crate::training_simulation::TrainingSimulation;
use crate::utils::set_panic_hook;
use neat_gru::train::train::Train;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

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

async fn run_training() {
    let mut sim = TrainingSimulation::new(700.0, 800.0);
    let mut runner: Train<TrainingSimulation, f64> = Train::new(&mut sim);
    runner
        .inputs(5)
        .outputs(2)
        .iterations(5000)
        .delta_threshold(3.0)
        .formula(4.0, 4.0, 4.0)
        .max_layers(100)
        .max_per_layers(100)
        .max_individuals(250)
        .start_async()
        .await;
}

#[wasm_bindgen]
pub fn start() {
    set_panic_hook();

    /*let space_pressed = Rc::new(RefCell::new(false));
    let pressed_clone = space_pressed.clone();

    let func = Closure::wrap(Box::new(move |js_event: web_sys::KeyboardEvent| {
        *pressed_clone.borrow_mut() = js_event.key_code() == 32;
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);*/

    /*document
    .add_event_listener_with_callback("keydown", func.as_ref().unchecked_ref())
    .unwrap();*/

    spawn_local(run_training());
}
