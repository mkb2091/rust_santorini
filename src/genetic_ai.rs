use crate::lib;
use rand::prelude::*;

lazy_static! {
    static ref GENES: [std::sync::Arc<dyn ActionScorer>; GENE_COUNT] = [
        std::sync::Arc::new(PrioritizeClimbing {}),
        std::sync::Arc::new(PrioritizeCapping {}),
    ];
}

const GENE_COUNT: usize = 2;

trait ActionScorer: Sync + Send {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32;
}

struct PrioritizeClimbing {}
impl ActionScorer for PrioritizeClimbing {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32 {
        let (w1, w2) = game.player_locations[player_id];
        let old_pos = if worker == lib::Worker::One { w1 } else { w2 };
        (game.board[movement.0 as usize][movement.1 as usize].to_int() as i32)
            - (game.board[old_pos.0 as usize][old_pos.1 as usize].to_int() as i32)
    }
}

struct PrioritizeCapping {}
impl ActionScorer for PrioritizeCapping {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32 {
        (game.board[build.0 as usize][build.1 as usize] == lib::TowerStates::Level3) as i32
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GeneticAI {
    gene_weighting: [u16; GENE_COUNT],
    //rng: rand::rngs::thread::ThreadRng
}

impl GeneticAI {
    pub fn new() -> Self {
        Self {
            gene_weighting: [10, 10],
        }
    }
}

impl GeneticAI {
    fn get_score(
        &mut self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32 {
        GENES
            .iter()
            .zip(self.gene_weighting.iter())
            .map(|(gene, weighting)| {
                gene.get_score(game, player_id, worker, movement, build) * (*weighting as i32)
            })
            .sum()
    }
    /**
    fn simplify(&mut self) {
        for i in [2]
            .iter()
            .chain(3..(self.gene_weighting.iter().min().unwrap_or(3)).step(2))
        {
            if self.gene_weighting.iter().all(|val| val % i == 0) {
                for val in self.gene_weighting.iter_mut() {
                    *val /= i;
                }
            }
        }
    }**/

    fn create_altered(&self, rng: &mut rand::rngs::ThreadRng) -> Self {
        let mut gene_weighting = self.gene_weighting.clone();
        let index = rng.gen_range(0, gene_weighting.len());
        gene_weighting[index] = gene_weighting[index].saturating_sub(1) + rng.gen_range(0, 2);
        Self { gene_weighting }
    }
}
impl lib::Player for GeneticAI {
    fn get_action(
        &mut self,
        game: &lib::Game,
        player_id: usize,
    ) -> (lib::Worker, (u8, u8), (u8, u8)) {
        *game
            .list_possible_actions(player_id)
            .iter()
            .max_by_key(|(worker, movement, build)| {
                self.get_score(game, player_id, *worker, *movement, *build)
            })
            .unwrap_or(&(lib::Worker::One, (0, 0), (0, 0)))
    }
}

pub fn train(
    base: Vec<GeneticAI>,
    max_length: usize,
    iterations: usize,
    matches: usize,
) -> Vec<GeneticAI> {
    let mut ais = base.clone();
    let mut rng = rand::thread_rng();
    let mut squares: Vec<(u8, u8)> = (0..25).map(|val| (val / 5, val % 5)).collect();
    for iteration in 0..iterations {
        println!("Training iteration: {}", iteration);
        let mut new: Vec<GeneticAI> = ais.iter().map(|ai| ai.create_altered(&mut rng)).collect();
        let (mut old_scores, mut new_scores) =
            (Vec::with_capacity(ais.len()), Vec::with_capacity(ais.len()));
        for _ in 0..ais.len() {
            old_scores.push(0);
            new_scores.push(0);
        }
        for (i1, ai1) in ais.iter().enumerate() {
            for (i2, ai2) in new.iter().enumerate() {
                for _ in 0..matches {
                    squares.shuffle(&mut rng);
                    let players: [Option<Box<(dyn lib::Player)>>; 3] =
                        [Some(Box::new(*ai1)), Some(Box::new(*ai2)), None];
                    let result = lib::GameManager::new(
                        players,
                        [
                            (squares[0], squares[1]),
                            (squares[2], squares[3]),
                            (squares[4], squares[5]),
                        ],
                    )
                    .main_loop();
                    if let Some(result) = result {
                        if result == 0 {
                            old_scores[i1] += 1
                        } else {
                            new_scores[i2] += 1
                        }
                    }
                }
            }
        }
        let mut total: Vec<(GeneticAI, usize)> = ais
            .into_iter()
            .zip(old_scores.into_iter())
            .chain(new.into_iter().zip(new_scores.into_iter()))
            .collect();
        total.sort_unstable_by_key(|(_, score)| *score);
        ais = total
            .into_iter()
            .map(|(ai, _)| ai)
            .take(max_length)
            .collect();
    }
    ais
}
