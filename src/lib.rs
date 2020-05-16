pub trait Player {
    fn get_action(
        &mut self,
        board: &[[TowerStates; 5]; 5],
        worker1: (u8, u8),
        worker2: (u8, u8),
        player_locations: [((u8, u8), (u8, u8)); 3],
        player_statuses: [Status; 3],
    ) -> (Worker, (u8, u8), (u8, u8));
}

#[derive(Debug, Copy, Clone)]
pub enum TowerStates {
    Empty,
    Level1,
    Level2,
    Level3,
    Capped,
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
    players: [Option<Box<dyn Player>>; 3],
    player_locations: [((u8, u8), (u8, u8)); 3],
    player_statuses: [Status; 3],
}

impl Game {
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
            board: [[TowerStates::Empty; 5]; 5],
            players,
            player_locations,
            player_statuses,
        }
    }
    pub fn main_loop(&mut self) -> usize {
        loop {
            let players: Vec<usize> = self
                .player_statuses
                .iter()
                .enumerate()
                .filter(|(_, &status)| status == Status::Playing)
                .map(|(i, _)| i)
                .collect();
            for &player_id in players.iter() {
                if let Some(player) = &mut self.players[player_id] {
                    print_board(self.board, self.player_locations, self.player_statuses);
                    println!("Player: {}", player_id);
                    let (worker1, worker2) = self.player_locations[player_id];
                    let (worker, (move_x, move_y), (build_x, build_y)) = player.get_action(
                        &self.board,
                        worker1,
                        worker2,
                        self.player_locations.clone(),
                        self.player_statuses.clone(),
                    );
                    for (w1, w2) in self.player_locations.iter() {
                        if (move_x, move_y) == *w1 || (move_x, move_y) == *w2 {
                            println!("Player moved into already occupied space");
                            self.player_statuses[player_id] = Status::Dead;
                        }
                    }
                    if self.player_statuses[player_id] == Status::Playing {
                        if worker == Worker::One
                            && (worker1.0 as i8 - move_x as i8).abs() <= 1
                            && (worker1.1 as i8 - move_y as i8).abs() <= 1
                        {
                            self.player_locations[player_id].0 = (move_x, move_y);
                        } else if worker == Worker::Two
                            && (worker2.0 as i8 - move_x as i8).abs() <= 1
                            && (worker2.1 as i8 - move_y as i8).abs() <= 1
                        {
                            self.player_locations[player_id].1 = (move_x, move_y);
                        } else {
                            println!("Player moved more than allowed distance");
                            self.player_statuses[player_id] = Status::Dead;
                        }
                    }
                }
            }
        }
    }
}

pub fn print_board(
    board: [[TowerStates; 5]; 5],
    player_locations: [((u8, u8), (u8, u8)); 3],
    player_statuses: [Status; 3],
) {
    let mut result = String::new();
    let mut new_board: [[Option<(Worker, usize)>; 5]; 5] = [[None; 5]; 5];
    for (player, ((w1x, w1y), (w2x, w2y))) in player_locations.iter().enumerate() {
        if player_statuses[player] == Status::Playing {
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
    for (i, (row1, row2)) in board.iter().zip(new_board.iter()).enumerate() {
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
