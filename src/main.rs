mod lib;

struct RealPlayer {
    name: String,
}

impl RealPlayer {
    pub fn new(name: String) -> Self {
        Self { name}
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
        println!("{:?}", board);
        (lib::Worker::One, (0, 0), (0, 0))
    }
}

fn main() {
    println!("Hello, world!");
}
