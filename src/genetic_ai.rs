use crate::action_score_algorithms;
use crate::lib;
use crate::start_location_score_algorithms;
use rand::prelude::*;
use rayon::prelude::*;

const GENE_COUNT: usize = 5;
const START_LOCATION_GENE_COUNT: usize = 3;
const TOTAL_PERMUTATIONS: usize = (3 * 3 * 3 * 3 * 3) * (3 * 3 * 3) - 1;
lazy_static! {
    static ref GENES: [std::sync::Arc<dyn ActionScorer>; GENE_COUNT] = [
        std::sync::Arc::new(action_score_algorithms::PrioritizeClimbing {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeCapping {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeBlocking {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeBuildingLow {}),
        std::sync::Arc::new(action_score_algorithms::PrioritizeNextToPlayer {}),
    ];
    static ref START_LOCATION_GENES: [std::sync::Arc<dyn StartScorer>; START_LOCATION_GENE_COUNT] = [
        std::sync::Arc::new(start_location_score_algorithms::StartNearPlayers {}),
        std::sync::Arc::new(start_location_score_algorithms::StartNearMiddle {}),
        std::sync::Arc::new(start_location_score_algorithms::StartAwayFromOtherWorker {}),
    ];
}

pub trait ActionScorer: Sync + Send {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
        is_near_player: bool,
        will_be_near_player: bool,
        will_build_near_player: bool,
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

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct GeneticAI {
    pub gene_weighting: [u16; GENE_COUNT],
    pub start_location_gene_weighting: [u16; START_LOCATION_GENE_COUNT],
    //rng: rand::rngs::thread::ThreadRng
}

impl GeneticAI {
    pub fn new() -> Self {
        Self {
            gene_weighting: [0, 0, 0, 0, 0],
            start_location_gene_weighting: [0, 0, 0],
        }
    }
}

impl GeneticAI {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
        is_near_player: bool,
    ) -> i32 {
        let will_be_near_player = game.is_near_player(player_id, movement);
        let will_build_near_player = game.is_near_player(player_id, build);
        GENES
            .iter()
            .zip(self.gene_weighting.iter())
            .filter(|(_, weighting)| **weighting != 0)
            .map(|(gene, weighting)| {
                gene.get_score(
                    game,
                    player_id,
                    worker,
                    movement,
                    build,
                    is_near_player,
                    will_be_near_player,
                    will_build_near_player,
                ) * (*weighting as i32)
            })
            .sum()
    }

    fn get_start_location_score(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> i32 {
        START_LOCATION_GENES
            .iter()
            .zip(self.start_location_gene_weighting.iter())
            .filter(|(_, weighting)| **weighting != 0)
            .map(|(gene, weighting)| {
                gene.get_score(player_locations, start_locations, other_starting_location)
                    * (*weighting as i32)
            })
            .sum()
    }
    fn simplify(&mut self) {
        for i in 2..(*self.gene_weighting.iter().min().unwrap_or(&3)) {
            if self.gene_weighting.iter().all(|val| val % i == 0) {
                for val in self.gene_weighting.iter_mut() {
                    *val /= i;
                }
            }
        }
    }
    fn create_random(rng: &mut rand::rngs::ThreadRng) -> Self {
        let gene_weighting = [
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
        ];
        let start_location_gene_weighting = [
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
        ];
        Self {
            gene_weighting,
            start_location_gene_weighting,
        }
    }

    fn create_altered(&self) -> [Self; TOTAL_PERMUTATIONS] {
        let amount: u16 = 1;
        let mut altered = [*self; TOTAL_PERMUTATIONS];
        let g = self.gene_weighting;
        let gsl = self.start_location_gene_weighting;
        let mut index = 0;
        for g0 in [
            g[0].saturating_sub(amount),
            g[0],
            g[0].saturating_add(amount),
        ]
        .iter()
        {
            for g1 in [
                g[1].saturating_sub(amount),
                g[1],
                g[1].saturating_add(amount),
            ]
            .iter()
            {
                for g2 in [
                    g[2].saturating_sub(amount),
                    g[2],
                    g[2].saturating_add(amount),
                ]
                .iter()
                {
                    for g3 in [
                        g[3].saturating_sub(amount),
                        g[3],
                        g[3].saturating_add(amount),
                    ]
                    .iter()
                    {
                        for g4 in [
                            g[4].saturating_sub(amount),
                            g[4],
                            g[4].saturating_add(amount),
                        ]
                        .iter()
                        {
                            for gsl0 in [
                                gsl[0].saturating_sub(amount),
                                gsl[0],
                                gsl[0].saturating_add(amount),
                            ]
                            .iter()
                            {
                                for gsl1 in [
                                    gsl[1].saturating_sub(amount),
                                    gsl[1],
                                    gsl[1].saturating_add(amount),
                                ]
                                .iter()
                                {
                                    for gsl2 in [
                                        gsl[2].saturating_sub(amount),
                                        gsl[2],
                                        gsl[2].saturating_add(amount),
                                    ]
                                    .iter()
                                    {
                                        let new = Self {
                                            gene_weighting: [*g0, *g1, *g2, *g3, *g4],
                                            start_location_gene_weighting: [*gsl0, *gsl1, *gsl2],
                                        };
                                        if *self != new {
                                            altered[index] = new;
                                            index += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        altered
    }
}
impl lib::Player for GeneticAI {
    fn get_action(&self, game: &lib::Game, player_id: usize) -> lib::Action {
        let actions = game.list_possible_actions(player_id);
        if actions.is_empty() {
            (lib::Worker::One, (0, 0), (0, 0))
        } else {
            let location = game.player_locations[player_id];
            let w1_is_near_player = game.is_near_player(player_id, location.0);
            let w2_is_near_player = game.is_near_player(player_id, location.1);
            let action_scores = actions
                .iter()
                .map(|(worker, movement, build)| {
                    ((*worker, *movement, *build), {
                        self.get_score(
                            game,
                            player_id,
                            *worker,
                            *movement,
                            *build,
                            if *worker == lib::Worker::One {
                                w1_is_near_player
                            } else {
                                w2_is_near_player
                            },
                        )
                    })
                })
                .collect::<Vec<(lib::Action, i32)>>();
            if let Some((_, highest_score)) = action_scores.iter().max_by_key(|(_, score)| score) {
                let options = action_scores
                    .iter()
                    .filter(|(_, score)| score == highest_score)
                    .map(|(action, _)| *action)
                    .collect::<Vec<lib::Action>>();
                options[rand::thread_rng().gen_range(0, options.len())]
            } else {
                (lib::Worker::One, (0, 0), (0, 0))
            }
        }
    }
    fn get_starting_position(
        &self,
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
        (*first_location, *second_location)
    }
}

fn compare_ai(ai1: &dyn lib::Player, ai2: &dyn lib::Player, matches: usize) -> (usize, usize) {
    let mut scores = (0, 0);
    for _ in 0..matches {
        let players: [Option<&dyn lib::Player>; 3] = [Some(ai1), Some(ai2), None];
        let result = lib::main_loop(players);
        if let Some(result) = result {
            if result == 0 {
                scores.0 += 1
            } else {
                scores.1 += 1
            }
        }

        let players: [Option<&dyn lib::Player>; 3] = [Some(ai2), Some(ai1), None];
        let result = lib::main_loop(players);
        if let Some(result) = result {
            if result == 0 {
                scores.1 += 1
            } else {
                scores.0 += 1
            }
        }
    }
    scores
}

pub fn train(
    players: Vec<Box<(dyn lib::Player)>>,
    iterations: usize,
    matches: usize,
) -> Vec<GeneticAI> {
    let mut ais_for_testing: Vec<Box<(dyn lib::Player)>> = Vec::with_capacity(iterations);
    let mut ais: Vec<GeneticAI> = Vec::with_capacity(iterations);
    let mut rng = rand::thread_rng();

    for iteration in 0..iterations {
        let mut old_ai = GeneticAI::create_random(&mut rng);
        let mut old_score: usize = players
            .iter()
            .chain(ais_for_testing.iter())
            .map(|ai| compare_ai(&old_ai, &**ai, matches).0)
            .sum();
        let mut generations: usize = 0;
        loop {
            let altered = old_ai.create_altered();
            if let Some((better_ai, better_score)) = altered
                .par_iter()
                .map(|new_ai| {
                    (
                        new_ai,
                        players
                            .iter()
                            .chain(ais_for_testing.iter())
                            .map(|ai| compare_ai(new_ai, &**ai, matches).0)
                            .sum::<usize>(),
                    )
                })
                .filter(|(_, score)| *score > old_score)
                .max_by_key(|(_, score)| *score)
            {
                old_score = better_score;
                old_ai = *better_ai;
                generations += 1;
            } else {
                let accepted = old_score >= (players.len() + ais.len()) * matches;
                println!(
                    "{} new score {} (out of {}) at iteration {} after {} generations",
                    if accepted { "Accepted" } else { "Rejected" },
                    old_score,
                    (players.len() + ais.len()) * matches * 2,
                    iteration,
                    generations
                );
                if accepted {
                    ais_for_testing.push(Box::new(old_ai));
                    ais.push(old_ai);
                }
                break;
            }
        }
    }
    ais
}
