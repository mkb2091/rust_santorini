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
        build: (u8, u8),
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
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        build: (u8, u8),
    ) -> i32 {
        (game.board[build.0 as usize][build.1 as usize] == lib::TowerStates::Level3) as i32
    }
}

pub struct PrioritizeBlocking {}
impl ActionScorer for PrioritizeBlocking {
    fn get_score(
        &self,
        game: &lib::Game,
        player_id: usize,
        worker: lib::Worker,
        movement: (u8, u8),
        b: (u8, u8),
    ) -> i32 {
        let mut nearby_player = false;
        for (player, (w1, w2)) in game.player_locations.iter().enumerate() {
            if player != player_id {
                for &w in &[w1, w2] {
                    if (w.0 as i8 - b.0 as i8).abs() <= 1 && (w.1 as i8 - b.1 as i8).abs() <= 1 {
                        nearby_player = true;
                    }
                }
            }
        }
        if !nearby_player {
            0
        } else {
            let mut max_near_height = 0;
            for &s in &[
                (b.0 - 1, b.1 - 1),
                (b.0, b.1 - 1),
                (b.0 + 1, b.1 - 1),
                (b.0 - 1, b.1),
                (b.0 - 1, b.1 + 1),
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
                std::cmp::Ordering::Less => 0,
                std::cmp::Ordering::Equal => 1,
                std::cmp::Ordering::Greater => 2,
            }
        }
    }
}
