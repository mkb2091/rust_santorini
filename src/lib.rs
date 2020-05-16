pub trait Player {
    fn get_action(
        &mut self,
        board: &[[TowerStates; 5]; 5],
        worker1: (u8, u8),
        worker2: (u8, u8),
        player_locations: [((u8, u8), (u8, u8)); 3],
        player_statuses: [Status; 3],
    ) -> (Worker, (u8, u8), (u8, u8));
}

#[derive(Debug, Copy, Clone)]
pub enum TowerStates {
    Empty,
    Level1,
    Level2,
    Level3,
    Capped,
}

pub enum Worker {
    One,
    Two,
}

#[derive(PartialEq, Copy, Clone, )]
pub enum Status {
    Playing,
    Dead,
}

pub struct Game {
    board: [[TowerStates; 5]; 5],
    players: [Option<Box<dyn Player>>; 3],
    player_locations: [((u8, u8), (u8, u8)); 3],
    player_statuses: [Status; 3],
}

impl Game {
    pub fn new(
        players: [Option<Box<dyn Player>>; 3],
        player_locations: [((u8, u8), (u8, u8)); 3],
    ) -> Self {
        let mut player_statuses = [Status::Dead; 3];
        for (i, player) in players.iter().enumerate() {
            if player.is_some() {
                player_statuses[i] = Status::Playing;
            }
        }
        Self {
            board: [[TowerStates::Empty; 5]; 5],
            players,
            player_locations,
            player_statuses,
        }
    }
    pub fn main_loop(&mut self) -> usize {
        loop {
            for (i, player) in self
                .players
                .iter_mut().zip(self.player_statuses.iter())
                .enumerate()
                .filter(|(_, (_, status))| **status == Status::Playing)
            {
                if let (Some(player), _) = player {
                    let (worker1, worker2) = self.player_locations[i];
                    let (worker, movement, build) = player.get_action(
                        &self.board,
                        worker1,
                        worker2,
                        self.player_locations.clone(),
                        self.player_statuses.clone(),
                    );
                }
            }
        }
    }
}
