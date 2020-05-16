pub trait Player {
    fn get_action(&mut self, game: &Game, player_id: usize) -> (Worker, (u8, u8), (u8, u8));
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TowerStates {
    Empty,
    Level1,
    Level2,
    Level3,
    Capped,
}

impl TowerStates {
    pub fn increase(&self) -> Option<Self> {
        match self {
            TowerStates::Empty => Some(TowerStates::Level1),
            TowerStates::Level1 => Some(TowerStates::Level2),
            TowerStates::Level2 => Some(TowerStates::Level3),
            TowerStates::Level3 => Some(TowerStates::Capped),
            TowerStates::Capped => None,
        }
    }
    pub fn to_int(&self) -> u8 {
        match self {
            TowerStates::Empty => 0,
            TowerStates::Level1 => 1,
            TowerStates::Level2 => 2,
            TowerStates::Level3 => 3,
            TowerStates::Capped => 4,
        }
    }
}

impl ToString for TowerStates {
    fn to_string(&self) -> String {
        match self {
            TowerStates::Empty => "◌",
            TowerStates::Level1 => "○",
            TowerStates::Level2 => "◍",
            TowerStates::Level3 => "◉",
            TowerStates::Capped => "●",
        }
        .to_string()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Worker {
    One,
    Two,
}

impl ToString for Worker {
    fn to_string(&self) -> String {
        match self {
            Worker::One => "O",
            Worker::Two => "T",
        }
        .to_string()
    }
}
#[derive(PartialEq, Copy, Clone)]
pub enum Status {
    Playing,
    Dead,
}

pub struct Game {
    board: [[TowerStates; 5]; 5],
    player_locations: [((u8, u8), (u8, u8)); 3],
    player_statuses: [Status; 3],
}

impl Game {
    pub fn is_valid(
        &self,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> bool {
        let (move_x, move_y) = movement;
        let (build_x, build_y) = build;
        for (i, (w1, w2)) in self.player_locations.iter().enumerate() {
            if self.player_statuses[i] == Status::Playing {
                if (move_x, move_y) == *w1 || (move_x, move_y) == *w2 {
                    println!("Player moved into already occupied space");
                    return false;
                }
                if (worker == Worker::One && (build_x, build_y) == *w2)
                    || (worker == Worker::Two && (build_x, build_y) == *w1)
                {
                    println!("Cannot build on occupied block");
                    return false;
                }
            }
        }

        let base_worker = if worker == Worker::One {
            self.player_locations[player_id].0
        } else {
            self.player_locations[player_id].1
        };
        if (base_worker.0 as i8 - move_x as i8).abs() > 1
            && (base_worker.1 as i8 - move_y as i8).abs() > 1
        {
            // Check for moving more than 1 block
            println!("Workers cannot move more than one block");
            return false;
        } else if (self.board[base_worker.0 as usize][base_worker.1 as usize].to_int() as i8
            - self.board[move_x as usize][move_y as usize].to_int() as i8)
            < -1
        {
            // Check for moving up more than 1 level
            println!("Workers cannot move up more than one level higher");
            return false;
        } else if self.board[move_x as usize][move_y as usize] == TowerStates::Capped {
            // Check for moving on to a dome
            println!("Worker cannot move on to a dome");
            return false;
        } else if (move_x, move_y) == (build_x, build_y) {
            // Check for building where
            println!("Cannot build on occupied block");
            return false;
        } else if (move_x as i8 - build_x as i8).abs() > 1
            && (move_y as i8 - build_y as i8).abs() > 1
        {
            println!("Build location is not within range");
            return false;
        } else if self.board[build_x as usize][build_y as usize] == TowerStates::Capped {
            println!("Can't build on dome");
            return false;
        } else {
            return true;
        }
    }

    pub fn print_board(&self) {
        let mut result = String::new();
        let mut new_board: [[Option<(Worker, usize)>; 5]; 5] = [[None; 5]; 5];
        for (player, ((w1x, w1y), (w2x, w2y))) in self.player_locations.iter().enumerate() {
            if self.player_statuses[player] == Status::Playing {
                new_board[*w1x as usize][*w1y as usize] = Some((Worker::One, player));
                new_board[*w2x as usize][*w2y as usize] = Some((Worker::Two, player));
            }
        }
        result.push(' ');
        for i in 0..5 {
            result.push_str("  ");
            result.push_str(&i.to_string());
            result.push_str("   ");
        }
        result.push('\n');
        for (i, (row1, row2)) in self.board.iter().zip(new_board.iter()).enumerate() {
            result.push(' ');
            for square in row1.iter() {
                result.push(' ');
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push(' ');
            }
            result.push('\n');
            result.push_str(&i.to_string());
            for (s1, s2) in row1.iter().zip(row2.iter()) {
                result.push(' ');
                result.push_str(&s1.to_string());
                if let Some((worker, player)) = s2 {
                    result.push_str(&player.to_string());
                    result.push_str(&worker.to_string());
                } else {
                    result.push_str("  ");
                }
                result.push_str(&s1.to_string());
                result.push(' ');
            }
            result.push('\n');

            result.push(' ');
            for square in row1.iter() {
                result.push(' ');
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push_str(&square.to_string());
                result.push(' ');
            }
            result.push('\n');
            result.push('\n');
        }
        println!("Game:\n{}", result);
    }
    pub fn list_possible_actions(&self, player_id: usize) -> Vec<(Worker, (u8, u8), (u8, u8))> {
        let mut possible_actions: Vec<(Worker, (u8, u8), (u8, u8))> = Vec::new();
        for &worker in [Worker::One, Worker::Two].iter() {
            for move_x in 0..5 {
                for move_y in 0..5 {
                    for build_x in 0..5 {
                        for build_y in 0..5 {
                            if self.is_valid(player_id, worker, (move_x, move_y), (build_x, build_y)) {
                                possible_actions.push((
                                    worker,
                                    (move_x, move_y),
                                    (build_x, build_y),
                                ));
                            }
                        }
                    }
                }
            }
        }
possible_actions
    }
}

pub struct GameManager {
    game: Game,
    players: [Option<Box<dyn Player>>; 3],
}

impl GameManager {
    pub fn new(
        players: [Option<Box<dyn Player>>; 3],
        player_locations: [((u8, u8), (u8, u8)); 3],
    ) -> Self {
        let mut player_statuses = [Status::Dead; 3];
        for (i, player) in players.iter().enumerate() {
            if player.is_some() {
                player_statuses[i] = Status::Playing;
            }
        }
        Self {
            players,
            game: Game {
                board: [[TowerStates::Empty; 5]; 5],
                player_locations,
                player_statuses,
            },
        }
    }

    pub fn main_loop(&mut self) -> usize {
        loop {
            let players: Vec<usize> = self
                .game
                .player_statuses
                .iter()
                .enumerate()
                .filter(|(_, &status)| status == Status::Playing)
                .map(|(i, _)| i)
                .collect();
            for &player_id in players.iter() {
                if let Some(player) = &mut self.players[player_id] {
                    self.game.print_board();
                    println!("Player: {}", player_id);
                    let (worker, (move_x, move_y), (build_x, build_y)) =
                        player.get_action(&self.game, player_id);

                    if self
                        .game
                        .is_valid(player_id, worker, (move_x, move_y), (build_x, build_y))
                    {
                        if worker == Worker::One {
                            self.game.player_locations[player_id].0 = (move_x, move_y);
                        } else {
                            self.game.player_locations[player_id].1 = (move_x, move_y);
                        }

                        if let Some(new) =
                            self.game.board[build_x as usize][build_y as usize].increase()
                        {
                            self.game.board[build_x as usize][build_y as usize] = new;
                        }
                    }
                }
            }
        }
    }
}
