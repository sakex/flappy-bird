extern crate serde;

mod game;
mod training_simulation;
mod utils;

use crate::training_simulation::TrainingSimulation;
use crate::utils::set_panic_hook;
use neat_gru::train::Train;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

async fn run_training(params: GameParams) {
    let outputs_count = if params.game_type == 0 { 1 } else { 2 };
    let birds_count = params.birds_count;

    let mut sim = TrainingSimulation::new(700.0, 800.0, params);
    let mut runner: Train<TrainingSimulation, f64> = Train::new(&mut sim);

    runner
        .inputs(3)
        .outputs(outputs_count)
        .iterations(5000)
        .delta_threshold(2.)
        .formula(0.8, 0.8, 0.3)
        .max_layers(10)
        .max_per_layers(10)
        .max_individuals(birds_count as usize)
        .access_train_object(Box::new(|train| {
            let species_count = train.species_count();
            train.simulation.species_count = species_count;
        }))
        .start_async()
        .await;
}

#[wasm_bindgen]
pub struct GameParams {
    pub game_type: i32,
    pub birds_count: i32,
    pub render_count: i32,
    pub hole_size: i32,
}

#[wasm_bindgen]
impl GameParams {
    #[wasm_bindgen(constructor)]
    pub fn new(game_type: i32, birds_count: i32, render_count: i32, hole_size: i32) -> GameParams {
        GameParams {
            game_type,
            birds_count,
            render_count,
            hole_size,
        }
    }
}

impl Clone for GameParams {
    fn clone(&self) -> GameParams {
        GameParams {
            game_type: self.game_type,
            birds_count: self.birds_count,
            render_count: self.render_count,
            hole_size: self.hole_size,
        }
    }
}

#[wasm_bindgen]
pub fn start(params: GameParams) {
    set_panic_hook();
    let document = web_sys::window().unwrap().document().unwrap();
    let start_button = document.get_element_by_id("start").unwrap();
    start_button
        .set_attribute("style", "display: none;")
        .unwrap();
    spawn_local(run_training(params));
}
