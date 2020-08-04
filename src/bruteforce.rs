use crate::*;
use rand::prelude::*;

pub struct BruteForce {
    depth: usize,
}

impl BruteForce {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }

    fn evaluate_action(
        &self,
        mut game: Game,
        player_id: usize,
        action: Action,
        depth: usize,
    ) -> f32 {
        match game.apply_action(player_id, action, true) {
            Ok(true) => return 1.0,
            Ok(false) => {}
            Err(()) => return -1.0,
        };
        // Might need fixing/improving for when dealing with 3 player
        let next_player = (player_id + 1) % 3;
        let next_player = if game.player_statuses[next_player] == Status::Playing {
            next_player
        } else {
            (player_id + 2) % 3
        };
        if depth != 0 {
            let mut other_players_best_outcome = f32::MIN;
            for action2 in game.list_possible_actions(next_player).into_iter() {
                let score = self.evaluate_action(game, next_player, action2, depth - 1);
                if score > other_players_best_outcome {
                    if score == 1.0 {
                        break;
                    }
                    other_players_best_outcome = score;
                }
            }
            -other_players_best_outcome
        } else {
            let (w1, w2) = game.player_locations[player_id];
            let (nw1, nw2) = game.player_locations[next_player];
            (game.board[w1.0 as usize][w1.1 as usize].to_int() as i32
                + game.board[w2.0 as usize][w2.1 as usize].to_int() as i32
                - game.board[nw1.0 as usize][nw1.1 as usize].to_int() as i32
                - game.board[nw2.0 as usize][nw2.1 as usize].to_int() as i32) as f32
                * 0.1
        }
    }
}
impl Player for BruteForce {
    fn get_action(&self, game: &Game, player_id: usize) -> Action {
        let mut actions = (f32::MIN, Vec::new());
        for action in game.list_possible_actions(player_id).into_iter() {
            let score = self.evaluate_action(game.clone(), player_id, action, self.depth);
            if score > actions.0 {
                actions = (score, vec![action])
            } else if score == actions.0 {
                actions.1.push(action)
            }
        }
        actions
            .1
            .choose(&mut rand::thread_rng())
            .map(|action| *action)
            .unwrap_or((Worker::One, (0, 0), (0, 0)))
    }

    fn get_starting_position(&self, _: &Game, player_locations: &[StartLocation]) -> StartLocation {
        let mut values: Vec<(u8, u8)> = Vec::new();
        for &i in [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (1, 0)].iter() {
            if player_locations
                .iter()
                .all(|&(val1, val2)| val1 != i && val2 != i)
            {
                values.push(i);
            }
        }
        values.shuffle(&mut rand::thread_rng());
        (values[0], values[1])
    }
}
