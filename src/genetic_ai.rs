use crate::*;
use rand::prelude::*;
use rayon::prelude::*;

use rand::seq::SliceRandom;

const GENE_COUNT: usize = 5;
const START_LOCATION_GENE_COUNT: usize = 3;

const STEP_SIZE: f32 = 0.001;

const GENES: [&'static dyn ActionScorer; GENE_COUNT - 1] = [
    &action_score_algorithms::PrioritizeClimbing::new(),
    &action_score_algorithms::PrioritizeCapping::new(),
    &action_score_algorithms::PrioritizeBlocking::new(),
    &action_score_algorithms::PrioritizeNextToPlayer::new(),
];

const START_LOCATION_GENES: [&dyn StartScorer; START_LOCATION_GENE_COUNT] = [
    &start_location_score_algorithms::StartNearPlayers::new(),
    &start_location_score_algorithms::StartNearMiddle::new(),
    &start_location_score_algorithms::StartAwayFromOtherWorker::new(),
];

pub type TrainingData = (bool, usize, Game, Action);

pub trait ActionScorer: Sync + Send {
    fn get_score(
        &self,
        game: &Game,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        build: (u8, u8),
        is_near_player: bool,
        will_be_near_player: bool,
        will_build_near_player: bool,
    ) -> f32;
}

pub trait StartScorer: Sync + Send {
    fn get_score(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> f32;
}

pub trait ActivationFunction:
    std::marker::Sync + std::marker::Send + std::fmt::Debug + std::marker::Copy + Clone
{
    fn activation(x: f32) -> f32;
    fn inverse_activation(x: f32) -> f32;
    fn activation_derivative(x: f32) -> f32;
}

#[derive(Copy, Clone, Debug)]
pub struct Tanh {}
impl ActivationFunction for Tanh {
    fn activation(x: f32) -> f32 {
        x.tanh()
    }
    fn inverse_activation(x: f32) -> f32 {
        let result = if x.abs() > 0.9999 {
            (0.9999 as f32).atanh().copysign(x)
        } else {
            x.atanh()
        };
        assert!(result.is_finite(), "x: {}, result: {}", x, result);
        result
    }
    fn activation_derivative(x: f32) -> f32 {
        1.0 - x.tanh().powi(2)
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct GeneticAI<A: ActivationFunction> {
    pub gene_weighting: [f32; GENE_COUNT],
    pub start_location_gene_weighting: [f32; START_LOCATION_GENE_COUNT],
    phantom: std::marker::PhantomData<A>, //rng: rand::rngs::thread::ThreadRng
}

impl<A: ActivationFunction> GeneticAI<A> {
    pub fn new() -> Self {
        Self {
            gene_weighting: [1.0; GENE_COUNT],
            start_location_gene_weighting: [1.0; START_LOCATION_GENE_COUNT],
            phantom: std::marker::PhantomData,
        }
    }
}

impl<A: ActivationFunction> GeneticAI<A> {
    fn get_unprocessed(
        &self,
        game: &Game,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        build: (u8, u8),
        is_near_player: bool,
    ) -> [f32; GENE_COUNT] {
        let will_be_near_player = game.is_near_player(player_id, movement);
        let will_build_near_player = game.is_near_player(player_id, build);
        let mut output = [1.0; GENE_COUNT];
        for (ptr, gene) in output.iter_mut().zip(GENES.iter()) {
            *ptr = gene.get_score(
                game,
                player_id,
                worker,
                movement,
                build,
                is_near_player,
                will_be_near_player,
                will_build_near_player,
            )
        }
        output
    }

    fn get_score_from_unprocessed(&self, unprocessed: &[f32; GENE_COUNT]) -> f32 {
        A::activation(
            unprocessed
                .iter()
                .zip(self.gene_weighting.iter())
                .map(|(gene, weight)| gene * weight)
                .sum(),
        )
    }
    fn get_score(
        &self,
        game: &Game,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        build: (u8, u8),
        is_near_player: bool,
    ) -> f32 {
        // if game.board[movement.0 as usize][movement.1 as usize] == TowerStates::Level3 {
        //     return f32::MAX;
        // }
        self.get_score_from_unprocessed(&self.get_unprocessed(
            game,
            player_id,
            worker,
            movement,
            build,
            is_near_player,
        ))
    }

    fn get_start_location_score(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> f32 {
        START_LOCATION_GENES
            .iter()
            .zip(self.start_location_gene_weighting.iter())
            .filter(|(_, weighting)| **weighting != 0.0)
            .map(|(gene, weighting)| {
                gene.get_score(player_locations, start_locations, other_starting_location)
                    * weighting
            })
            .sum()
    }
    pub fn create_random(rng: &mut rand::rngs::ThreadRng) -> Self {
        let gene_weighting = [rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        let start_location_gene_weighting = [rng.gen(), rng.gen(), rng.gen()];
        Self {
            gene_weighting,
            start_location_gene_weighting,
            phantom: std::marker::PhantomData,
        }
    } //[0.94549954, 0.36510202, 0.18945187, 0.32395408, 0.06980309]

    pub fn get_action_gradients(
        &self,
        target_score: f32,
        unprocessed: &[f32; GENE_COUNT],
    ) -> [f32; GENE_COUNT] {
        let with_weights: f32 = unprocessed
            .iter()
            .zip(self.gene_weighting.iter())
            .map(|(gene, weight)| gene * weight)
            .sum();
        let output = A::activation(with_weights);
        // overall_score = (target_score - output).powi(2);

        // d(overall)/d(output) = 2.0 * (target_score - output);
        // d(output)/d(with_weights) = activation_derivative(with_weights);

        // d(overall)/d(with_weights) = d(output)/d(with_weights) * d(overall)/d(output)
        // d(overall)/d(with_weights) = activation_derivative(with_weights) * 2.0 * (target_score - output)

        // d(overall)/d(weights) = d(with_weights)/d(weights) * d(overall)/d(with_weights)
        // d(with_weights)/d(weights) = unprocessed
        // d(overall)/d(weights) = weights * (activation_derivative(result) * 2.0 * (target_score - output))
        let d_overall_d_with_weights =
            A::activation_derivative(with_weights) * 2.0 * (target_score - output);
        let mut gradients = *unprocessed;
        for ptr in gradients.iter_mut() {
            *ptr *= d_overall_d_with_weights;
        }
        gradients
    }

    pub fn learn(&mut self, results: &[TrainingData], iterations: usize) {
        let mut training_data: Vec<(f32, [f32; GENE_COUNT])> = Vec::new();
        for (success, player_id, game, (worker, movement, build)) in results.iter() {
            // Generate more training data
            // for (success, (worker, movement, build)) in game
            //     .list_possible_actions(*player_id)
            //     .into_iter()
            //     .filter(|other| other != action && false)
            //     .map(|item| (!success, item))
            //     .chain(std::iter::once((*success, *action)))
            // {}
            let unprocessed = self.get_unprocessed(
                game,
                *player_id,
                *worker,
                *movement,
                *build,
                game.is_near_player(
                    *player_id,
                    match *worker {
                        Worker::One => game.player_locations[*player_id as usize].0,
                        Worker::Two => game.player_locations[*player_id as usize].1,
                    },
                ),
            );
            training_data.push((if *success { 1.0 } else { -1.0 }, unprocessed));
        }
        debug_assert!({
            println!("Learning from {} actions", training_data.len());
            true
        });
        fn get_overall_score<A: ActivationFunction>(
            ai: &GeneticAI<A>,
            training_data: &[(f32, [f32; GENE_COUNT])],
        ) -> f64 {
            training_data
                .iter()
                .map(|(target_score, unprocessed)| {
                    (*target_score as f64 - ai.get_score_from_unprocessed(unprocessed) as f64)
                        .powi(2)
                })
                .sum()
        }
        let mut total_score_before: f64 = get_overall_score(self, &training_data);

        for i in 0..iterations {
            let mut overall_gradient = [0.0; GENE_COUNT];
            for (target_score, unprocessed) in training_data.iter() {
                for (ptr, new) in overall_gradient
                    .iter_mut()
                    .zip(self.get_action_gradients(*target_score, unprocessed).iter())
                {
                    *ptr += new;
                }
            }
            let mut new = *self;
            for (ptr, gradient) in new.gene_weighting.iter_mut().zip(overall_gradient.iter()) {
                *ptr += gradient * STEP_SIZE;
            }
            let new_score = get_overall_score(&new, &training_data);
            if new_score < total_score_before {
                *self = new;
                println!(
                    "{}: Successfully trained from {} to {}",
                    i, total_score_before, new_score
                );
                total_score_before = new_score
            } else {
                println!(
                    "{}: Failed training from {} up to {}",
                    i, total_score_before, new_score
                );
                println!("Gradient was: {:?}", overall_gradient);
                break;
            }
        }
    }
    pub fn self_train(&mut self, iterations: usize, batch_size: usize) {
        for iteration in 0..iterations {
            if iteration % 10 == 0 {
                println!("Iteration: {}", iteration);
            }

            let mut results: Vec<TrainingData> = Vec::new();
            for _ in 0..batch_size {
                let mut action_history: Option<[Vec<(Game, Action)>; 3]> =
                    Some([vec![], vec![], vec![]]);
                let tmp_players: [Option<&dyn Player>; 3] = [Some(self), Some(self), None];
                let result = main_loop(tmp_players, false, &mut action_history, &mut None);
                for (player_id, action_list) in action_history.unwrap().iter().enumerate() {
                    for (game, action) in action_list.iter() {
                        results.push((player_id == result, player_id, *game, *action));
                    }
                }
            }
            self.learn(&results, 100);
        }
    }
    pub fn train(&mut self, players: Vec<Box<(dyn Player)>>, iterations: usize, batch_size: usize) {
        let mut total_win_count = 0;
        for iteration in 0..iterations {
            let mut win_count = 0;
            let mut results: Vec<TrainingData> = Vec::new();
            for _ in 0..batch_size {
                for player in players.iter() {
                    let mut action_history1: Option<[Vec<(Game, Action)>; 3]> =
                        Some([vec![], vec![], vec![]]);
                    let tmp_players: [Option<&dyn Player>; 3] = [Some(self), Some(&**player), None];
                    let result1 = main_loop(tmp_players, false, &mut action_history1, &mut None);
                    if result1 == 0 {
                        win_count += 1;
                    }

                    let mut action_history2: Option<[Vec<(Game, Action)>; 3]> =
                        Some([vec![], vec![], vec![]]);
                    let tmp_players: [Option<&dyn Player>; 3] = [Some(&**player), Some(self), None];
                    let result2 = main_loop(tmp_players, false, &mut action_history2, &mut None);
                    if result2 == 1 {
                        win_count += 1;
                    }
                    for (result, action_history) in
                        [(result1, action_history1), (result2, action_history2)].iter()
                    {
                        for (player_id, action_list) in
                            action_history.as_ref().unwrap().iter().enumerate()
                        {
                            for (game, action) in action_list.iter() {
                                results.push((player_id == *result, player_id, *game, *action));
                            }
                        }
                    }
                }
            }
            total_win_count += win_count;
            self.learn(&results, 100);
            println!(
                "Iteration: {}, wins: {}, total_wins: {}",
                iteration,
                (win_count as f32) / ((batch_size * players.len() * 2) as f32),
                (total_win_count as f32)
                    / ((batch_size * players.len() * (iteration + 1) * 2) as f32)
            );
        }
    }
}
impl<A: ActivationFunction> Player for GeneticAI<A> {
    fn get_action(&self, game: &Game, player_id: usize) -> Action {
        let actions = game.list_possible_actions(player_id);
        if actions.is_empty() {
            (Worker::One, (0, 0), (0, 0))
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
                            if *worker == Worker::One {
                                w1_is_near_player
                            } else {
                                w2_is_near_player
                            },
                        )
                    })
                })
                .collect::<Vec<(Action, f32)>>();

            let mut max = f32::MIN;
            for (_, score) in action_scores.iter() {
                if *score > max {
                    max = *score;
                }
            }
            if max == f32::MIN {
                return (Worker::One, (0, 0), (0, 0));
            }
            let options = action_scores
                .iter()
                .filter(|(_, score)| *score == max)
                .map(|(action, _)| *action)
                .collect::<Vec<Action>>();
            *options.choose(&mut rand::thread_rng()).unwrap()
        }
    }
    fn get_starting_position(
        &self,
        _: &Game,
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
        let first_start_location_scores = values
            .iter()
            .map(|location| {
                (
                    *location,
                    self.get_start_location_score(player_locations, *location, None),
                )
            })
            .collect::<Vec<_>>();
        let mut max = ((0, 0), f32::MIN);
        for (action, score) in first_start_location_scores.iter() {
            if *score > max.1 {
                max = (*action, *score);
            }
        }
        let options = first_start_location_scores
            .iter()
            .filter(|(_, score)| *score == max.1)
            .map(|(action, _)| *action)
            .collect::<Vec<(u8, u8)>>();
        let first_location = *options.choose(&mut rand::thread_rng()).unwrap();

        let second_start_location_scores = values
            .iter()
            .filter(|location| **location != first_location)
            .map(|location| {
                (
                    *location,
                    self.get_start_location_score(
                        player_locations,
                        *location,
                        Some(first_location),
                    ),
                )
            })
            .collect::<Vec<_>>();
        let mut max = ((0, 0), f32::MIN);
        for (action, score) in second_start_location_scores.iter() {
            if *score > max.1 {
                max = (*action, *score);
            }
        }
        let options = second_start_location_scores
            .iter()
            .filter(|(_, score)| *score == max.1)
            .map(|(action, _)| *action)
            .collect::<Vec<(u8, u8)>>();
        let second_location = *options.choose(&mut rand::thread_rng()).unwrap();
        (first_location, second_location)
    }
}

fn compare_ai(ai1: &dyn Player, ai2: &dyn Player, matches: usize) -> (usize, usize) {
    let mut scores = (0, 0);
    for _ in 0..matches {
        let players: [Option<&dyn Player>; 3] = [Some(ai1), Some(ai2), None];
        let result = main_loop(players, false, &mut None, &mut None);
        if result == 0 {
            scores.0 += 1
        } else {
            scores.1 += 1
        }

        let players: [Option<&dyn Player>; 3] = [Some(ai2), Some(ai1), None];
        let result = main_loop(players, false, &mut None, &mut None);
        if result == 0 {
            scores.1 += 1
        } else {
            scores.0 += 1
        }
    }
    scores
}
