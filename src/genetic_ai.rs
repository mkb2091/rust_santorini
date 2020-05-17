use crate::lib;
use crate::action_score_algorithms;
use crate::start_location_score_algorithms;
use rand::prelude::*;

const GENE_COUNT: usize = 3;
const START_LOCATION_GENE_COUNT: usize = 1;
lazy_static! {
    static ref GENES: [std::sync::Arc<dyn ActionScorer>; GENE_COUNT] = [
        std::sync::Arc::new(action_score_algorithms::PrioritizeClimbing {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeCapping {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeBlocking {}),
    ];
    static ref START_LOCATION_GENES: [std::sync::Arc<dyn StartScorer>; START_LOCATION_GENE_COUNT] =
        [std::sync::Arc::new(start_location_score_algorithms::StartNearPlayers {}),];
}

pub trait ActionScorer: Sync + Send {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32;
}

pub trait StartScorer: Sync + Send {
    fn get_score(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> i32;
}

struct StartNearPlayers {}
impl StartScorer for StartNearPlayers {
    fn get_score(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        s: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> i32 {
        -player_locations
            .iter()
            .map(|(w1, w2)| {
                ((w1.0 as i8 - s.0 as i8)
                    .abs()
                    .max((w1.1 as i8 - s.1 as i8).abs())
                    + (w2.0 as i8 - s.0 as i8)
                        .abs()
                        .max((w2.1 as i8 - s.1 as i8).abs())) as i32
            })
            .sum::<i32>()
    }
}



#[derive(PartialEq, Copy, Clone, Debug)]
pub struct GeneticAI {
    pub gene_weighting: [u16; GENE_COUNT],
    pub start_location_gene_weighting: [u16; START_LOCATION_GENE_COUNT],
    //rng: rand::rngs::thread::ThreadRng
}

impl GeneticAI {
    pub fn new() -> Self {
        Self {
            gene_weighting: [0, 0, 0],
            start_location_gene_weighting: [0],
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

    fn get_start_location_score(
        &mut self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> i32 {
        START_LOCATION_GENES
            .iter()
            .zip(self.start_location_gene_weighting.iter())
            .map(|(gene, weighting)| {
                gene.get_score(player_locations, start_locations, other_starting_location)
                    * (*weighting as i32)
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
        let gene_weighting = {
            let mut gene_weighting = self.gene_weighting.clone();
            let index = rng.gen_range(0, gene_weighting.len());
            gene_weighting[index] = gene_weighting[index].saturating_sub(1) + rng.gen_range(0, 2);
            gene_weighting
        };
        let start_location_gene_weighting = {
            let mut start_location_gene_weighting = self.start_location_gene_weighting.clone();
            let index = rng.gen_range(0, start_location_gene_weighting.len());
            start_location_gene_weighting[index] =
                start_location_gene_weighting[index].saturating_sub(1) + rng.gen_range(0, 2);
            start_location_gene_weighting
        };
        Self {
            gene_weighting,
            start_location_gene_weighting,
        }
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
    fn get_starting_position(
        &mut self,

        _: &lib::Game,
        player_locations: &[((u8, u8), (u8, u8))],
    ) -> ((u8, u8), (u8, u8)) {
        let mut values: Vec<(u8, u8)> = Vec::new();
        for i in (0..25).map(|val| (val / 5, val % 5)) {
            if player_locations
                .iter()
                .all(|&(val1, val2)| val1 != i && val2 != i)
            {
                values.push(i);
            }
        }
        let first_location = values
            .iter()
            .max_by_key(|location| {
                self.get_start_location_score(player_locations, **location, None)
            })
            .unwrap_or(&values[0]);
        let second_location = values
            .iter()
            .filter(|x| *x != first_location)
            .max_by_key(|location| {
                self.get_start_location_score(player_locations, **location, Some(*first_location))
            })
            .unwrap_or(&values[0]);
        return (*first_location, *second_location);
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
                    let players: [Option<Box<(dyn lib::Player)>>; 3] =
                        [Some(Box::new(*ai1)), Some(Box::new(*ai2)), None];
                    let result = lib::GameManager::new(players).main_loop();
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
