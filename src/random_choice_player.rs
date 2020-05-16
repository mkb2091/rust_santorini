use crate::lib;
use rand::prelude::*;

pub struct RandomChoice {}

impl RandomChoice {
    pub fn new() -> Self {
        Self {}
    }
}
impl lib::Player for RandomChoice {
    fn get_action(
        &mut self,
        game: &lib::Game,
        player_id: usize,
    ) -> (lib::Worker, (u8, u8), (u8, u8)) {
        let mut possible_actions = game.list_possible_actions(player_id);
        if !possible_actions.is_empty() {
            possible_actions.shuffle(&mut rand::thread_rng());
            possible_actions[0]
        } else {
            (lib::Worker::One, (0, 0), (0, 0))
        }
    }
}
