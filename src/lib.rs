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
        .inputs(3)
        .outputs(2)
        .iterations(5000)
        .delta_threshold(2.)
        .formula(0.8, 0.8, 0.3)
        .max_layers(10)
        .max_per_layers(10)
        .max_individuals(500)
        .access_train_object(Box::new(|train| {
            let species_count = train.species_count();
            train.simulation.species_count = species_count;
        }))
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
