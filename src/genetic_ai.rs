use crate::*;
use rand::prelude::*;
use rayon::prelude::*;

use rand::seq::SliceRandom;

const GENE_COUNT: usize = 4;
const START_LOCATION_GENE_COUNT: usize = 3;

const STEP_SIZE: f32 = 0.001;

const GENES: [&'static dyn ActionScorer; GENE_COUNT] = [
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

#[derive(PartialEq, Clone, Debug)]
pub struct GeneticAI<A: nn::ActivationFunction> {
    pub gene_weighting: nn::NeuralNet<A>,
    pub start_location_gene_weighting: nn::NeuralNet<A>,
}

impl<A: nn::ActivationFunction> GeneticAI<A> {
    pub fn new() -> Self {
        Self {
            gene_weighting: nn::NeuralNet::new(GENE_COUNT),
            start_location_gene_weighting: nn::NeuralNet::new(GENE_COUNT),
        }
    }
}

impl<A: nn::ActivationFunction> GeneticAI<A> {
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
    
    
    fn get_unprocessed_starting_location(
        &self,
        player_locations: &[((u8, u8), (u8, u8))],
        start_locations: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> [f32; START_LOCATION_GENE_COUNT] {
        let mut output = [1.0; START_LOCATION_GENE_COUNT];
        for (ptr, gene) in output.iter_mut().zip(START_LOCATION_GENES.iter()) {
            *ptr = 
                gene.get_score(player_locations, start_locations, other_starting_location);
        }
        output
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
        self.gene_weighting.predict(&self.get_unprocessed(
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
        self.start_location_gene_weighting.predict(&self.get_unprocessed_starting_location(player_locations, start_locations, other_starting_location))
    }
        
    pub fn create_random(rng: &mut rand::rngs::ThreadRng) -> Self {
        Self {
            gene_weighting: nn::NeuralNet::create_random(GENE_COUNT, rng),
            start_location_gene_weighting: nn::NeuralNet::create_random(START_LOCATION_GENE_COUNT, rng),
        }
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
        self.gene_weighting.learn(&training_data, iterations, STEP_SIZE);
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
impl<A: nn::ActivationFunction> Player for GeneticAI<A> {
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

