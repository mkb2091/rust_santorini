mod lib;

struct RealPlayer {
    name: String,
}

impl RealPlayer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl lib::Player for RealPlayer {
    fn get_action(
        &mut self,
        board: &[[lib::TowerStates; 5]; 5],
        worker1: (u8, u8),
        worker2: (u8, u8),
        player_locations: [((u8, u8), (u8, u8)); 3],
        player_statuses: [lib::Status; 3],
    ) -> (lib::Worker, (u8, u8), (u8, u8)) {
        (lib::Worker::One, (0, 0), (0, 0))
    }
}

fn main() {
    let player1 = RealPlayer::new("1".to_string());
    let player2 = RealPlayer::new("2".to_string());
    let players: [Option<Box<(dyn lib::Player)>>; 3] =
        [Some(Box::new(player1)), Some(Box::new(player2)), None];
    let mut game = lib::Game::new(
        players,
        [((1, 1), (1, 2)), ((3, 0), (2, 4)), ((0, 0), (0, 0))],
    );
    game.main_loop();
    println!("Hello, world!");
}
