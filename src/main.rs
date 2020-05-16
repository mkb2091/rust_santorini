mod lib;
mod random_choice_player;

struct RealPlayer {}

impl RealPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl lib::Player for RealPlayer {
    fn get_action(
        &mut self,
        game: &lib::Game,
        player_id: usize,
    ) -> (lib::Worker, (u8, u8), (u8, u8)) {
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
}

fn main() {
    let player1 = RealPlayer::new();
    let player2 = random_choice_player::RandomChoice::new();
    let players: [Option<Box<(dyn lib::Player)>>; 3] =
        [Some(Box::new(player1)), Some(Box::new(player2)), None];
    let mut game = lib::GameManager::new(
        players,
        [((1, 1), (1, 2)), ((3, 0), (2, 4)), ((0, 0), (0, 0))],
    );
    game.main_loop();
    println!("Hello, world!");
}
