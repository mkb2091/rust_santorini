use crate::genetic_ai::ActionScorer;
use crate::lib;

pub struct PrioritizeClimbing {}
impl ActionScorer for PrioritizeClimbing {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        _build: (u8, u8),
        _is_near_player: bool,
        _will_be_near_player: bool,
        _will_build_near_player: bool,
    ) -> i32 {
        let (w1, w2) = game.player_locations[player_id];
        let old_pos = if worker == lib::Worker::One { w1 } else { w2 };
        (game.board[movement.0 as usize][movement.1 as usize].to_int() as i32)
            - (game.board[old_pos.0 as usize][old_pos.1 as usize].to_int() as i32)
    }
}

pub struct PrioritizeCapping {}
impl ActionScorer for PrioritizeCapping {
    fn get_score(
        &self,
        game: &lib::Game,
        _player_id: usize,
        _worker: lib::Worker,
        _movement: (u8, u8),
        b: (u8, u8),
        _is_near_player: bool,
        _will_be_near_player: bool,
        will_build_near_player: bool,
    ) -> i32 {
        if game.board[b.0 as usize][b.1 as usize] == lib::TowerStates::Level3 {
            if will_build_near_player {
                1
            } else {
                -1
            }
        } else if will_build_near_player {
            -1
        } else {
            0
        }
    }
}

pub struct PrioritizeBlocking {}
impl ActionScorer for PrioritizeBlocking {
    fn get_score(
        &self,
        game: &lib::Game,
        _player_id: usize,
        _worker: lib::Worker,
        _movement: (u8, u8),
        b: (u8, u8),
        is_near_player: bool,
        _will_be_near_player: bool,
        _will_build_near_player: bool,
    ) -> i32 {
        if !is_near_player {
            0
        } else {
            let mut max_near_height = 0;
            for &s in &[
                (b.0.wrapping_sub(1), b.1.wrapping_sub(1)),
                (b.0, b.1.wrapping_sub(1)),
                (b.0 + 1, b.1.wrapping_sub(1)),
                (b.0.wrapping_sub(1), b.1),
                (b.0.wrapping_sub(1), b.1 + 1),
                (b.0, b.1 + 1),
                (b.0 + 1, b.1),
                (b.0 + 1, b.1 + 1),
            ] {
                if s.0 <= 4 && s.1 <= 4 {
                    max_near_height =
                        max_near_height.max(game.board[s.0 as usize][s.1 as usize].to_int());
                }
            }

            let current_height = game.board[b.0 as usize][b.1 as usize].to_int();
            match current_height.cmp(&max_near_height) {
                std::cmp::Ordering::Less => -2,
                std::cmp::Ordering::Equal => -1,
                std::cmp::Ordering::Greater => 1,
            }
        }
    }
}

pub struct PrioritizeNextToPlayer {}
impl ActionScorer for PrioritizeNextToPlayer {
    fn get_score(
        &self,
        _game: &lib::Game,
        _player_id: usize,
        _worker: lib::Worker,
        _m: (u8, u8),
        _build: (u8, u8),
        is_near_player: bool,
        will_be_near_player: bool,
        _will_build_near_player: bool,
    ) -> i32 {
        if is_near_player {
            if will_be_near_player {
                0
            } else {
                -1
            }
        } else if will_be_near_player {
            1
        } else {
            -1
        }
    }
}
