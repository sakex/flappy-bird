use crate::game::game::Game;
use crate::GameParams;
use async_trait::async_trait;
use neat_gru::game::GameAsync;
use neat_gru::neural_network::NeuralNetwork;
use neat_gru::topology::Topology;

pub struct TrainingSimulation {
    width: f64,
    height: f64,
    params: GameParams,
    networks: Option<Vec<NeuralNetwork<f64>>>,
    generation: usize,
    pub species_count: usize,
}

unsafe impl Send for TrainingSimulation {}
unsafe impl Sync for TrainingSimulation {}

impl TrainingSimulation {
    pub fn new(width: f64, height: f64, params: GameParams) -> TrainingSimulation {
        TrainingSimulation {
            width,
            height,
            params,
            generation: 0,
            networks: None,
            species_count: 1,
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
        let generation = self.generation;
        self.generation += 1;
        let width = self.width;
        let height = self.height;
        let species_count = self.species_count;
        let render_count = self.params.render_count;
        let hole_size = self.params.hole_size;
        let networks = self.networks.take().unwrap();
        match self.params.game_type {
            0 => {
                let game = Game::<0>::run_game(
                    width,
                    height,
                    render_count,
                    species_count,
                    generation,
                    hole_size,
                    networks,
                )
                .await;
                let lock = &*game.lock().unwrap();
                lock.scores.clone()
            }
            1 => {
                let game = Game::<1>::run_game(
                    width,
                    height,
                    render_count,
                    species_count,
                    generation,
                    hole_size,
                    networks,
                )
                .await;
                let lock = &*game.lock().unwrap();
                lock.scores.clone()
            }
            _ => {
                panic!("Invalid game type")
            }
        }
    }
}
