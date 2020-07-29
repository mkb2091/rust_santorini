use crate::*;

pub struct FirstChoice {}

impl FirstChoice {
    pub fn new() -> Self {
        Self {}
    }
}
impl Player for FirstChoice {
    fn get_action(&self, game: &Game, player_id: usize) -> (Worker, (u8, u8), (u8, u8)) {
        let possible_actions = game.list_possible_actions(player_id);
        if !possible_actions.is_empty() {
            possible_actions[0]
        } else {
            (Worker::One, (0, 0), (0, 0))
        }
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
        (values[0], values[1])
    }
}
