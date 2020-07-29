use crate::genetic_ai::StartScorer;
use crate::*;

pub struct StartNearPlayers {}
impl StartNearPlayers {
    pub const fn new() -> Self {
        Self {}
    }
}
impl StartScorer for StartNearPlayers {
    fn get_score(
        &self,
        player_locations: &[StartLocation],
        s: (u8, u8),
        _other_starting_location: Option<(u8, u8)>,
    ) -> f32 {
        -player_locations
            .iter()
            .map(|(w1, w2)| {
                ((w1.0 as i8 - s.0 as i8)
                    .abs()
                    .max((w1.1 as i8 - s.1 as i8).abs())
                    + (w2.0 as i8 - s.0 as i8)
                        .abs()
                        .max((w2.1 as i8 - s.1 as i8).abs())) as f32
            })
            .sum::<f32>()
    }
}

pub struct StartNearMiddle {}
impl StartNearMiddle {
    pub const fn new() -> Self {
        Self {}
    }
}
impl StartScorer for StartNearMiddle {
    fn get_score(
        &self,
        _player_locations: &[StartLocation],
        s: (u8, u8),
        _other_starting_location: Option<(u8, u8)>,
    ) -> f32 {
        -(s.0 as i8 - 2).max(s.1 as i8 - 2).abs() as f32
    }
}

pub struct StartAwayFromOtherWorker {}
impl StartAwayFromOtherWorker {
    pub const fn new() -> Self {
        Self {}
    }
}
impl StartScorer for StartAwayFromOtherWorker {
    fn get_score(
        &self,
        _player_locations: &[StartLocation],
        s: (u8, u8),
        other_starting_location: Option<(u8, u8)>,
    ) -> f32 {
        if let Some(ow) = other_starting_location {
            (s.0 as i8 - ow.0 as i8).max(s.1 as i8 - ow.0 as i8).abs() as f32
        } else {
            0.0
        }
    }
}
