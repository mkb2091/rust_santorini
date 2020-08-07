use rust_santorini::*;

use rand::Rng;
use std::io::BufRead;
use std::io::Write;

struct RealPlayer {}

impl RealPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for RealPlayer {
    fn get_action(&self, game: &Game, player_id: usize) -> Action {
        println!("Player: {}", player_id);
        let possible_actions = game.list_possible_actions(player_id);
        if possible_actions.is_empty() {
            println!("No possible moves left");
            return (Worker::One, (0, 0), (0, 0));
        }
        loop {
            let worker: Worker = {
                println!("Enter which worker to select");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => println!("Error: {}", error),
                }
                match &input.trim().to_lowercase() as &str {
                    "o" => Worker::One,
                    "one" => Worker::One,
                    "1" => Worker::One,
                    "t" => Worker::Two,
                    "two" => Worker::Two,
                    "2" => Worker::Two,
                    _ => {
                        println!("Not a valid worker, can be either one or two");
                        continue;
                    }
                }
            };
            if !possible_actions.iter().any(|(w, _, _)| *w == worker) {
                println!("No possible moves with the chosen worker");
                continue;
            }
            let (move_x, move_y) = {
                println!("Enter coordinates of where to move worker seperated by a space");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => println!("Error: {}", error),
                }
                let coordinates: Vec<&str> = input.split_whitespace().collect();
                let (mut x, mut y) = (0, 0);
                let mut valid = false;
                if coordinates.len() == 2 {
                    if let Ok(x_tmp) = coordinates[1].parse::<u8>() {
                        if let Ok(y_tmp) = coordinates[0].parse::<u8>() {
                            x = x_tmp;
                            y = y_tmp;
                            valid = true;
                        }
                    };
                }
                if !valid {
                    println!("Not valid coordinates");
                    continue;
                } else {
                    (x, y)
                }
            };
            if !possible_actions
                .iter()
                .any(|(w, m, _)| (*w, *m) == (worker, (move_x, move_y)))
            {
                println!("Worker cannot move to the chosen square");
                continue;
            }
            let (build_x, build_y) = {
                println!("Enter coordinates of where to build seperated by a space");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => println!("Error: {}", error),
                }
                let coordinates: Vec<&str> = input.split_whitespace().collect();
                let (mut x, mut y) = (0, 0);
                let mut valid = false;
                if coordinates.len() == 2 {
                    if let Ok(x_tmp) = coordinates[1].parse::<u8>() {
                        if let Ok(y_tmp) = coordinates[0].parse::<u8>() {
                            x = x_tmp;
                            y = y_tmp;
                            valid = true;
                        }
                    };
                }
                if !valid {
                    println!("Not valid coordinates");
                    continue;
                } else {
                    (x, y)
                }
            };

            if !possible_actions
                .iter()
                .any(|(w, m, b)| (*w, *m, *b) == (worker, (move_x, move_y), (build_x, build_y)))
            {
                println!(
                    "Worker cannot move to the chosen square and then build at the chosen square"
                );
                continue;
            }
            return (worker, (move_x, move_y), (build_x, build_y));
        }
    }
    fn get_starting_position(
        &self,
        game: &Game,
        player_locations: &[StartLocation],
    ) -> StartLocation {
        game.print_board();
        loop {
            let w1 = {
                println!("Enter coordinates of first worker");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => println!("Error: {}", error),
                }
                let coordinates: Vec<&str> = input.split_whitespace().collect();
                let (mut x, mut y) = (0, 0);
                let mut valid = false;
                if coordinates.len() == 2 {
                    if let Ok(x_tmp) = coordinates[1].parse::<u8>() {
                        if let Ok(y_tmp) = coordinates[0].parse::<u8>() {
                            x = x_tmp;
                            y = y_tmp;
                            valid = true;
                        }
                    };
                }
                if !valid
                    || x > 4
                    || y > 4
                    || player_locations
                        .iter()
                        .any(|&(val1, val2)| val1 == (x, y) || val2 == (x, y))
                {
                    println!("Not valid coordinates");
                    continue;
                } else {
                    (x, y)
                }
            };
            let w2 = {
                println!("Enter coordinates of second worker");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(error) => println!("Error: {}", error),
                }
                let coordinates: Vec<&str> = input.split_whitespace().collect();
                let (mut x, mut y) = (0, 0);
                let mut valid = false;
                if coordinates.len() == 2 {
                    if let Ok(x_tmp) = coordinates[1].parse::<u8>() {
                        if let Ok(y_tmp) = coordinates[0].parse::<u8>() {
                            x = x_tmp;
                            y = y_tmp;
                            valid = true;
                        }
                    };
                }
                if !valid
                    || x > 4
                    || y > 4
                    || player_locations
                        .iter()
                        .any(|&(val1, val2)| val1 == (x, y) || val2 == (x, y))
                {
                    println!("Not valid coordinates");
                    continue;
                } else {
                    (x, y)
                }
            };
            return (w1, w2);
        }
    }
}

fn main() {
    let mut line = String::new();
    let mut training_data: Vec<genetic_ai::TrainingData> = Vec::new();
    if let Ok(file) = std::fs::File::open("training_data.json") {
        let mut buf = std::io::BufReader::new(file);
        while buf.read_line(&mut line).unwrap() > 0 {
            if let Ok(deserialized) = serde_json::from_str(&line) {
                training_data.push(deserialized);
            } else {
                println!("Failed to parse: {}", line);
            }
            line.clear();
        }
    } else {
        println!("Failed to load training data");
    }

    let mut start_location_training_data: Vec<_> = Vec::new();
    if let Ok(file) = std::fs::File::open("start_location_training_data.json") {
        let mut buf = std::io::BufReader::new(file);
        while buf.read_line(&mut line).unwrap() > 0 {
            if let Ok(deserialized) = serde_json::from_str(&line) {
                start_location_training_data.push(deserialized);
            } else {
                println!("Failed to parse: {}", line);
            }
            line.clear();
        }
    } else {
        println!("Failed to load training data");
    }

    let mut new_ai = genetic_ai::GeneticAI::<nn::Tanh>::create_random(&mut rand::thread_rng());
    new_ai.learn(&training_data, 1000);

    let mut game_predictor = game_prediction::GamePrediction::<nn::Tanh>::create_random(&mut rand::thread_rng());
    game_predictor.learn(
        &training_data
            .iter()
            .cloned()
            .map(|(success, player_id, game, _action)| (game, player_id, success))
            .collect::<Vec<_>>(),
        100000,
        0.0001,
    );
    println!("{:?}", game_predictor);
    // new_ai.self_train(100, 5);
    // new_ai.learn(&training_data);
    // new_ai.train(
    //     vec![Box::new(random_choice_player::RandomChoice::new())],
    //     100,
    //     10,
    // );
    // new_ai.train(
    //     vec![Box::new(genetic_ai::GeneticAI::<nn::Tanh>::new())],
    //     100,
    //     10,
    // );
    // println!("{:?}", new_ai);
    // let player2: &dyn Player = &RealPlayer::new();

    let player1: &dyn Player = &new_ai;
    let player2: &dyn Player = &bruteforce::BruteForce::new(game_predictor, 3, -0.9);

    let mut action_history: Option<_> = Some([vec![], vec![], vec![]]);
    let mut start_location_history: Option<_> = Some([None, None, None]);

    let mut scores = [0, 0];
    for round in 0..1 {
        let player1_first = rand::thread_rng().gen::<bool>();
        let players: [Option<&dyn Player>; 3] = if player1_first {
            [Some(player1), Some(player2), None]
        } else {
            [Some(player2), Some(player1), None]
        };
        let result = main_loop(players, false, &mut None, &mut None);
        scores[if player1_first { result } else { 1 - result }] += 1;
        println!("Scores: {:?}", scores);
    }
    let player1: &dyn Player = &RealPlayer::new();
    let players: [Option<&dyn Player>; 3] = if rand::thread_rng().gen::<bool>() {
        [Some(player1), Some(player2), None]
    } else {
        [Some(player2), Some(player1), None]
    };
    let result = main_loop(
        players,
        true,
        &mut action_history,
        &mut start_location_history,
    );
    println!("Player {} won the game", result);
    for (player_id, action_list) in action_history.unwrap().iter().enumerate() {
        for (game, action) in action_list.into_iter() {
            training_data.push((player_id == result, player_id, *game, *action));
        }
    }

    for (player_id, start_data) in start_location_history.unwrap().iter().enumerate() {
        if let Some((game, start_locations)) = start_data {
            start_location_training_data.push((
                player_id == result,
                *game,
                start_locations.clone(),
            ));
        }
    }
    if let Ok(file) = std::fs::File::create("training_data.json") {
        let mut buf = std::io::LineWriter::new(file);

        for td in training_data.iter() {
            buf.write(&serde_json::to_string(&td).unwrap().as_bytes())
                .unwrap();
            buf.write(&[b'\n']).unwrap();
        }
    } else {
        println!("Failed to write training data");
    }

    if let Ok(file) = std::fs::File::create("start_location_training_data.json") {
        let mut buf = std::io::LineWriter::new(file);

        for sd in start_location_training_data.iter() {
            buf.write(&serde_json::to_string(&sd).unwrap().as_bytes())
                .unwrap();
            buf.write(&[b'\n']).unwrap();
        }
    } else {
        println!("Failed to write start location training data");
    }
}
