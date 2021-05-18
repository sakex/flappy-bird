extern crate serde;

mod game;
mod training_simulation;
mod utils;

use crate::training_simulation::TrainingSimulation;
use crate::utils::set_panic_hook;
use neat_gru::train::train::Train;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

async fn run_training(params: GameParams) {
    let mut sim = TrainingSimulation::new(700.0, 800.0, params);
    let mut runner: Train<TrainingSimulation, f64> = Train::new(&mut sim);
    runner
        .inputs(3)
        .outputs(1)
        .iterations(5000)
        .delta_threshold(1.0)
        .formula(0.4, 0.4, 0.8)
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
pub struct GameParams {
    pub game_type: i32
}

#[wasm_bindgen]
impl GameParams {
    #[wasm_bindgen(constructor)]
    pub fn new(game_type: i32) -> GameParams {
        GameParams {
            game_type
        }
    }
}

impl Clone for GameParams {
    fn clone(&self) -> GameParams {
        GameParams {
            game_type: self.game_type
        }
    }
}

#[wasm_bindgen]
pub fn start(params: GameParams) {
    set_panic_hook();

    spawn_local(run_training(params));
}
