pub type Action = (Worker, (u8, u8), (u8, u8));
pub type StartLocation = ((u8, u8), (u8, u8));

pub trait Player: Send + Sync {
    fn get_action(&self, game: &Game, player_id: usize) -> Action;

    fn get_starting_position(
        &self,

        game: &Game,
        player_locations: &[((u8, u8), (u8, u8))],
    ) -> StartLocation;
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
    pub fn increase(self) -> Option<Self> {
        match self {
            TowerStates::Empty => Some(TowerStates::Level1),
            TowerStates::Level1 => Some(TowerStates::Level2),
            TowerStates::Level2 => Some(TowerStates::Level3),
            TowerStates::Level3 => Some(TowerStates::Capped),
            TowerStates::Capped => None,
        }
    }
    pub fn to_int(self) -> u8 {
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

impl std::cmp::PartialOrd for TowerStates {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_int().cmp(&other.to_int()))
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

#[derive(Copy, Clone)]
pub struct Game {
    pub board: [[TowerStates; 5]; 5],
    pub player_locations: [((u8, u8), (u8, u8)); 3],
    pub player_statuses: [Status; 3],
}

impl Game {
    pub fn can_move_to_square(&self, player_id: usize, worker: Worker, movement: (u8, u8)) -> bool {
        let (move_x, move_y) = movement;
        let base_worker = if worker == Worker::One {
            self.player_locations[player_id].0
        } else {
            self.player_locations[player_id].1
        };

        move_x <= 4
            && move_y <= 4
            && self
                .player_locations
                .iter()
                .enumerate()
                .filter(|(i, _)| self.player_statuses[*i] == Status::Playing)
                .all(|(_, (w1, w2))| movement != *w1 && movement != *w2)
            && (self.board[base_worker.0 as usize][base_worker.1 as usize].to_int() as i8
                - self.board[move_x as usize][move_y as usize].to_int() as i8)
                >= -1
            && movement != base_worker
            && (base_worker.0 as i8 - move_x as i8).abs() <= 1
            && (base_worker.1 as i8 - move_y as i8).abs() <= 1
        //&& self.board[move_x as usize][move_y as usize] != TowerStates::Capped
        // Not needed as to get onto a capped tower, would have to already be at level 3, which wins the game.
    }
    pub fn is_valid(
        &self,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        build: (u8, u8),
        checked_movement: bool,
    ) -> bool {
        let (move_x, move_y) = movement;
        let (build_x, build_y) = build;

        let (old_w1, old_w2) = self.player_locations[player_id];

        build_x <= 4
            && build_y <= 4
            && ((worker == Worker::One && build != old_w2)
                || (worker == Worker::Two && build != old_w1))
            && (checked_movement || self.can_move_to_square(player_id, worker, movement))
            && self
                .player_locations
                .iter()
                .enumerate()
                .filter(|(i, _)| self.player_statuses[*i] == Status::Playing && *i != player_id)
                .all(|(_, (w1, w2))| (build != *w1 && build != *w2))
            && movement != build
            && (move_x as i8 - build_x as i8).abs() <= 1
            && (move_y as i8 - build_y as i8).abs() <= 1
            && self.board[build_x as usize][build_y as usize] != TowerStates::Capped
    }

    pub fn print_board(&self) {
        let mut result = String::new();
        let mut new_board: [[Option<(Worker, usize)>; 5]; 5] = [[None; 5]; 5];
        for (player, &((w1x, w1y), (w2x, w2y))) in self.player_locations.iter().enumerate() {
            if self.player_statuses[player] == Status::Playing
                && (w1x <= 4 && w1y <= 4 && w2x <= 4 && w2y <= 4)
            {
                new_board[w1x as usize][w1y as usize] = Some((Worker::One, player));
                new_board[w2x as usize][w2y as usize] = Some((Worker::Two, player));
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
        println!("Levels: ◌○◍◉●\nGame:\n{}", result);
    }
    pub fn list_possible_actions(&self, player_id: usize) -> Vec<Action> {
        let mut possible_actions: Vec<Action> = Vec::with_capacity(2 * 8 * 8);
        let (worker1, worker2) = self.player_locations[player_id];
        for (&worker, &w) in [Worker::One, Worker::Two]
            .iter()
            .zip([worker1, worker2].iter())
        {
            for &m in [
                (w.0.wrapping_sub(1), w.1.wrapping_sub(1)),
                (w.0, w.1.wrapping_sub(1)),
                (w.0 + 1, w.1.wrapping_sub(1)),
                (w.0.wrapping_sub(1), w.1),
                (w.0.wrapping_sub(1), w.1 + 1),
                (w.0, w.1 + 1),
                (w.0 + 1, w.1),
                (w.0 + 1, w.1 + 1),
            ]
            .iter()
            .filter(|m| self.can_move_to_square(player_id, worker, **m))
            {
                for &b in &[
                    (m.0.wrapping_sub(1), m.1.wrapping_sub(1)),
                    (m.0, m.1.wrapping_sub(1)),
                    (m.0 + 1, m.1.wrapping_sub(1)),
                    (m.0.wrapping_sub(1), m.1),
                    (m.0.wrapping_sub(1), m.1 + 1),
                    (m.0, m.1 + 1),
                    (m.0 + 1, m.1),
                    (m.0 + 1, m.1 + 1),
                ] {
                    if self.is_valid(player_id, worker, m, b, true) {
                        possible_actions.push((worker, m, b));
                    }
                }
            }
        }
        possible_actions
    }
    pub fn is_near_player(&self, player_id: usize, pos: (u8, u8)) -> bool {
        self.player_locations
            .iter()
            .enumerate()
            .filter(|(i, _)| self.player_statuses[*i] == Status::Playing && *i != player_id)
            .any(|(_, (w1, w2))| {
                ((w1.0 as i8 - pos.0 as i8).abs() <= 1 && (w1.1 as i8 - pos.1 as i8).abs() <= 1)
                    || ((w2.0 as i8 - pos.0 as i8).abs() <= 1
                        && (w2.1 as i8 - pos.1 as i8).abs() <= 1)
            })
    }
}

pub fn main_loop(player_controls: [Option<&dyn Player>; 3]) -> Option<usize> {
    let mut game = Game {
        board: [[TowerStates::Empty; 5]; 5],
        player_locations: [((17, 17), (17, 17)); 3],
        player_statuses: {
            let mut player_statuses = [Status::Dead; 3];
            for (i, player) in player_controls.iter().enumerate() {
                if player.is_some() {
                    player_statuses[i] = Status::Playing;
                }
            }
            player_statuses
        },
    };
    let mut start_locations: Vec<((u8, u8), (u8, u8))> = Vec::new();
    let players: Vec<usize> = game
        .player_statuses
        .iter()
        .enumerate()
        .filter(|(_, &status)| status == Status::Playing)
        .map(|(i, _)| i)
        .collect();
    for &player_id in players.iter() {
        if let Some(player) = &player_controls[player_id] {
            loop {
                let (w1, w2) = player.get_starting_position(&game, &start_locations);
                if w1 != w2
                    && start_locations
                        .iter()
                        .all(|&(val1, val2)| !(w1 == val1 || w2 == val2 || w1 == val2))
                    && w1.0 <= 4
                    && w1.1 <= 4
                    && w2.0 <= 4
                    && w2.1 <= 4
                {
                    start_locations.push((w1, w2));
                    game.player_locations[player_id] = (w1, w2);
                    break;
                } else {
                    println!("Failed to enter valid start location: ({:?}, {:?})", w1, w2);
                }
            }
        }
    }

    while game
        .player_statuses
        .iter()
        .any(|&status| status == Status::Playing)
    {
        let players: Vec<usize> = game
            .player_statuses
            .iter()
            .enumerate()
            .filter(|(_, &status)| status == Status::Playing)
            .map(|(i, _)| i)
            .collect();
        for &player_id in players.iter() {
            if let Some(player) = &player_controls[player_id] {
                let (worker, (move_x, move_y), (build_x, build_y)) =
                    player.get_action(&game, player_id);

                if game.is_valid(
                    player_id,
                    worker,
                    (move_x, move_y),
                    (build_x, build_y),
                    false,
                ) {
                    if game.board[move_x as usize][move_y as usize] == TowerStates::Level3 {
                        //println!("Player {} won", player_id);
                        //game.print_board();
                        return Some(player_id);
                    }
                    if worker == Worker::One {
                        game.player_locations[player_id].0 = (move_x, move_y);
                    } else {
                        game.player_locations[player_id].1 = (move_x, move_y);
                    }

                    if let Some(new) = game.board[build_x as usize][build_y as usize].increase() {
                        game.board[build_x as usize][build_y as usize] = new;
                    }
                } else {
                    //println!("Not a valid move");
                    game.player_statuses[player_id] = Status::Dead;
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_worker_has_64_actions() {
        let game = Game {
            board: [[TowerStates::Empty; 5]; 5],
            player_locations: [
                ((2, 2), (17, 17)),
                ((17, 17), (17, 17)),
                ((17, 17), (17, 17)),
            ],
            player_statuses: [Status::Playing, Status::Dead, Status::Dead],
        };
        assert_eq!(game.list_possible_actions(0).len(), 64);
    }
    #[test]
    fn is_near_player_returns_false_when_not() {
        let game = Game {
            board: [[TowerStates::Empty; 5]; 5],
            player_locations: [((2, 2), (2, 3)), ((17, 17), (17, 17)), ((17, 17), (17, 17))],
            player_statuses: [Status::Playing, Status::Dead, Status::Dead],
        };
        assert_eq!(game.is_near_player(0, (3, 4)), false);
    }
    #[test]
    fn is_near_player_returns_true_when_near_player() {
        let game = Game {
            board: [[TowerStates::Empty; 5]; 5],
            player_locations: [((2, 2), (2, 3)), ((4, 2), (4, 3)), ((17, 17), (17, 17))],
            player_statuses: [Status::Playing, Status::Playing, Status::Dead],
        };
        assert_eq!(game.is_near_player(0, (3, 4)), true);
    }
    #[test]
    fn is_near_player_returns_false_when_only_near_player_is_dead() {
        let game = Game {
            board: [[TowerStates::Empty; 5]; 5],
            player_locations: [((2, 2), (2, 3)), ((4, 2), (4, 3)), ((17, 17), (17, 17))],
            player_statuses: [Status::Playing, Status::Dead, Status::Dead],
        };
        assert_eq!(game.is_near_player(0, (3, 4)), false);
    }
}
