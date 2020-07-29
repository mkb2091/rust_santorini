use crate::genetic_ai::ActionScorer;
use crate::*;

pub struct PrioritizeClimbing {}
impl PrioritizeClimbing {
    pub const fn new() -> Self {
        Self {}
    }
}
impl ActionScorer for PrioritizeClimbing {
    fn get_score(
        &self,
        game: &Game,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        _build: (u8, u8),
        _is_near_player: bool,
        _will_be_near_player: bool,
        _will_build_near_player: bool,
    ) -> f32 {
        let (w1, w2) = game.player_locations[player_id];
        let old_pos = if worker == Worker::One { w1 } else { w2 };
        (game.board[movement.0 as usize][movement.1 as usize].to_int() as f32)
            - (game.board[old_pos.0 as usize][old_pos.1 as usize].to_int() as f32)
    }
}

pub struct PrioritizeCapping {}
impl PrioritizeCapping {
    pub const fn new() -> Self {
        Self {}
    }
}
impl ActionScorer for PrioritizeCapping {
    fn get_score(
        &self,
        game: &Game,
        _player_id: usize,
        _worker: Worker,
        _movement: (u8, u8),
        b: (u8, u8),
        _is_near_player: bool,
        _will_be_near_player: bool,
        will_build_near_player: bool,
    ) -> f32 {
        if game.board[b.0 as usize][b.1 as usize] == TowerStates::Level3 {
            if will_build_near_player {
                1.0
            } else {
                -1.0
            }
        } else if will_build_near_player {
            -1.0
        } else {
            0.0
        }
    }
}

pub struct PrioritizeBlocking {}
impl PrioritizeBlocking {
    pub const fn new() -> Self {
        Self {}
    }
}
impl ActionScorer for PrioritizeBlocking {
    fn get_score(
        &self,
        game: &Game,
        player_id: usize,
        worker: Worker,
        movement: (u8, u8),
        b: (u8, u8),
        _is_near_player: bool,
        _will_be_near_player: bool,
        will_build_near_player: bool,
    ) -> f32 {
        if will_build_near_player {
            let mut max_near_height = 0;
            let (mut w1, mut w2) = game.player_locations[player_id];
            if worker == Worker::One {
                w1 = movement;
            } else {
                w2 = movement;
            }
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
                if s.0 <= 4 && s.1 <= 4 && s != w1 && s != w2 {
                    // Don't include the square players workers are on
                    max_near_height =
                        max_near_height.max(game.board[s.0 as usize][s.1 as usize].to_int());
                }
            }

            let current_height = game.board[b.0 as usize][b.1 as usize].to_int();
            match current_height as i8 - max_near_height as i8 {
                3 => -2.0,  // No need to dome towers that aren't surrounded
                2 => -2.0,  // See above
                1 => 2.0,   // Will result in blocking access
                0 => -3.0,  // Makes a step up for opponents
                -1 => -2.0, // Making a same height building for opponents
                _ => -1.0,  // No harm or benefit from filling in hole
            }
        } else {
            0.0
        }
    }
}

pub struct PrioritizeNextToPlayer {}
impl PrioritizeNextToPlayer {
    pub const fn new() -> Self {
        Self {}
    }
}
impl ActionScorer for PrioritizeNextToPlayer {
    fn get_score(
        &self,
        _game: &Game,
        _player_id: usize,
        _worker: Worker,
        _m: (u8, u8),
        _build: (u8, u8),
        is_near_player: bool,
        will_be_near_player: bool,
        _will_build_near_player: bool,
    ) -> f32 {
        if is_near_player {
            if will_be_near_player {
                0.0
            } else {
                -1.0
            }
        } else if will_be_near_player {
            1.0
        } else {
            -1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, Status, TowerStates};
    const TSE: TowerStates = TowerStates::Empty;
    const TS1: TowerStates = TowerStates::Level1;

    #[test]
    fn prioritize_climbing_scores_climb_highest() {
        let game = Game {
            board: [
                [TSE, TS1, TSE, TSE, TSE],
                [TSE; 5],
                [TSE; 5],
                [TSE; 5],
                [TSE; 5],
            ],
            player_locations: [((0, 0), (3, 3)), ((17, 17), (17, 17)), ((17, 17), (17, 17))],

            player_statuses: [Status::Playing, Status::Dead, Status::Dead],
        };
        let climbing = PrioritizeClimbing {};
        let actions = game.list_possible_actions(0);
        let mut max = ((Worker::One, (0, 0), (0, 0)), f32::MIN);
        for action in actions.iter() {
            let score =
                climbing.get_score(&game, 0, action.0, action.1, action.2, false, false, false);
            if score > max.1 {
                max = (*action, score);
            }
        }
        assert_eq!((max.0).1, (0, 1));
    }
    #[test]
    fn prioritize_climbing_scores_drop_lowest() {
        let game = Game {
            board: [
                [TS1, TSE, TS1, TS1, TS1],
                [TS1; 5],
                [TS1; 5],
                [TS1; 5],
                [TS1; 5],
            ],
            player_locations: [((0, 0), (3, 3)), ((17, 17), (17, 17)), ((17, 17), (17, 17))],

            player_statuses: [Status::Playing, Status::Dead, Status::Dead],
        };
        let climbing = PrioritizeClimbing {};
        let actions = game.list_possible_actions(0);
        let mut max = ((Worker::One, (0, 0), (0, 0)), f32::MAX);
        for action in actions.iter() {
            let score =
                climbing.get_score(&game, 0, action.0, action.1, action.2, false, false, false);
            if score < max.1 {
                max = (*action, score);
            }
        }
        assert_eq!((max.0).1, (0, 1));
    }
}
