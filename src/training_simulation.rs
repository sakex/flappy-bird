use crate::game::game::Game;
use async_trait::async_trait;
use neat_gru::game::GameAsync;
use neat_gru::neural_network::nn::NeuralNetwork;
use neat_gru::topology::topology::Topology;
use std::sync::Arc;
use wasm_bindgen::__rt::std::sync::Mutex;

pub struct TrainingSimulation {
    width: f64,
    height: f64,
    networks: Option<Vec<NeuralNetwork<f64>>>,
}

unsafe impl Send for TrainingSimulation {}
unsafe impl Sync for TrainingSimulation {}

impl TrainingSimulation {
    pub fn new(width: f64, height: f64) -> TrainingSimulation {
        TrainingSimulation {
            width,
            height,
            networks: None,
        }
    }
}

impl neat_gru::game::Game<f64> for TrainingSimulation {
    fn run_generation(&mut self) -> Vec<f64> {
        Vec::new()
    }

    fn reset_players(&mut self, nets: Vec<NeuralNetwork<f64>>) {
        self.networks = Some(nets);
    }

    fn post_training(&mut self, _: &[Topology<f64>]) {}
}

#[async_trait]
impl GameAsync<f64> for TrainingSimulation {
    async fn run_generation_async(&mut self) -> Vec<f64> {
        let width = self.width;
        let height = self.height;
        let networks = self.networks.take().unwrap();
        let game = Game::run_game(width, height, networks).await;
        //Game::sleep_until_ended(game.clone()).await;
        //TimeoutFuture::new(100).await;

        /*loop {
            TimeoutFuture::new(100).await;
            if game.lock().unwrap().ended() {
                break;
            }
        }*/
        let lock = &*game.lock().unwrap();
        let scores = lock.scores.clone();
        scores
    }
}
