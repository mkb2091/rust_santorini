use crate::genetic_ai::StartScorer;
use crate::lib;

pub struct StartNearPlayers {}
impl StartScorer for StartNearPlayers {
    fn get_score(
        &self,
        player_locations: &[lib::StartLocation],
        s: (u8, u8),
        _other_starting_location: Option<(u8, u8)>,
    ) -> i32 {
        -player_locations
            .iter()
            .map(|(w1, w2)| {
                ((w1.0 as i8 - s.0 as i8)
                    .abs()
                    .max((w1.1 as i8 - s.1 as i8).abs())
                    + (w2.0 as i8 - s.0 as i8)
                        .abs()
                        .max((w2.1 as i8 - s.1 as i8).abs())) as i32
            })
            .sum::<i32>()
    }
}
