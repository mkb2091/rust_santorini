mod action_score_algorithms;
mod first_choice_player;
mod genetic_ai;
mod lib;
mod random_choice_player;
mod start_location_score_algorithms;

#[macro_use]
extern crate lazy_static;

struct RealPlayer {}

impl RealPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl lib::Player for RealPlayer {
    fn get_action(&self, game: &lib::Game, player_id: usize) -> (lib::Worker, (u8, u8), (u8, u8)) {
        game.print_board();
        println!("Player: {}", player_id);
        loop {
            let worker: lib::Worker = {
                println!("Enter which worker to select");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(n) => {}
                    Err(error) => println!("Error: {}", error),
                }
                match &input.trim().to_lowercase() as &str {
                    "o" => lib::Worker::One,
                    "one" => lib::Worker::One,
                    "1" => lib::Worker::One,
                    "t" => lib::Worker::Two,
                    "two" => lib::Worker::Two,
                    "2" => lib::Worker::Two,
                    _ => {
                        println!("Not a valid worker, can be either one or two");
                        continue;
                    }
                }
            };
            let (move_x, move_y) = {
                println!("Enter coordinates of where to move worker seperated by a space");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(n) => {}
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
            let (build_x, build_y) = {
                println!("Enter coordinates of where to build seperated by a space");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(n) => {}
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
            return (worker, (move_x, move_y), (build_x, build_y));
        }
    }
    fn get_starting_position(
        &self,
        game: &lib::Game,
        player_locations: &[((u8, u8), (u8, u8))],
    ) -> ((u8, u8), (u8, u8)) {
        game.print_board();
        loop {
            let w1 = {
                println!("Enter coordinates of first worker");
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(n) => {}
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
                    Ok(n) => {}
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
    //let player1 = RealPlayer::new();
    let players: Vec<Box<(dyn lib::Player)>> = vec![
        Box::new(random_choice_player::RandomChoice::new()),
        Box::new(first_choice_player::FirstChoice::new()),
    ];
    let result = genetic_ai::train(players, 50, 1);
    println!("{:?}", result[0]);
    let player1: Box<(dyn lib::Player)> = Box::new(RealPlayer::new());
    let player2: Box<(dyn lib::Player)> = Box::new(result[0]);
    let players: [Option<&Box<(dyn lib::Player)>>; 3] = [Some(&player1), Some(&player2), None];
    lib::main_loop(players);
}
