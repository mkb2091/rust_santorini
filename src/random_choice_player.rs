use crate::lib;
use rand::prelude::*;

pub struct RandomChoice {}

impl RandomChoice {
    pub fn new() -> Self {
        Self {}
    }
}
impl lib::Player for RandomChoice {
    fn get_action(&self, game: &lib::Game, player_id: usize) -> lib::Action {
        let mut possible_actions = game.list_possible_actions(player_id);
        if !possible_actions.is_empty() {
            possible_actions.shuffle(&mut rand::thread_rng());
            possible_actions[0]
        } else {
            (lib::Worker::One, (0, 0), (0, 0))
        }
    }

    fn get_starting_position(
        &self,

        _: &lib::Game,
        player_locations: &[lib::StartLocation],
    ) -> lib::StartLocation {
        let mut values: Vec<(u8, u8)> = Vec::new();
        for &i in [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (1, 0)].iter() {
            if player_locations
                .iter()
                .all(|&(val1, val2)| val1 != i && val2 != i)
            {
                values.push(i);
            }
        }
        (values[0], values[1])
    }
}
