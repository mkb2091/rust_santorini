use crate::*;

const INPUT_COUNT: usize = 6;

#[derive(Debug, Clone)]
pub struct GamePrediction<A: nn::ActivationFunction> {
    network: nn::NeuralNet<A>,
}

fn get_total_height(game: &Game, player_id: usize) -> u8 {
    let (w1, w2) = game.player_locations[player_id];
    if w1.0 >= 5 || w1.1 >= 5 || w2.0 >= 5 || w2.1 >= 5 {
        0
    } else {
        game.board[w1.0 as usize][w1.1 as usize].to_int()
            + game.board[w2.0 as usize][w2.1 as usize].to_int()
    }
}

fn get_max_height(game: &Game, player_id: usize) -> u8 {
    let (w1, w2) = game.player_locations[player_id];
    if w1.0 >= 5 || w1.1 >= 5 || w2.0 >= 5 || w2.1 >= 5 {
        0
    } else {
        game.board[w1.0 as usize][w1.1 as usize]
            .to_int()
            .max(game.board[w2.0 as usize][w2.1 as usize].to_int())
    }
}

fn squares_movable_to(game: &Game, player_id: usize) -> u8 {
    let mut count = 0;
    let (w1, w2) = game.player_locations[player_id];
    for &((wx, wy), worker) in &[(w1, Worker::One), (w2, Worker::Two)] {
        if wx >= 5 || wy >= 5 {
            continue;
        }
        for &mx in &[wx.saturating_sub(1), wx, wx + 1] {
            if mx < 5 {
                for &my in &[wy.saturating_sub(1), wy, wy + 1] {
                    if my < 5 {
                        if game.can_move_to_square(player_id, worker, (mx, my)) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

impl<A: nn::ActivationFunction> GamePrediction<A> {
    pub fn new() -> Self {
        Self {
            network: nn::NeuralNet::new(INPUT_COUNT),
        }
    }
	
	pub fn create_random(rng: &mut rand::rngs::ThreadRng) -> Self {
        Self {
            network: nn::NeuralNet::create_random(INPUT_COUNT, rng),
        }
    }
	
	

    fn generate_input(&self, game: &Game, player_id: usize) -> [f32; INPUT_COUNT] {
        let mut input = [0.0; INPUT_COUNT];
        input[0] = game.can_win_on_next_turn(player_id) as u8 as f32;
        input[1] = game.can_win_on_next_turn((player_id + 1) % 3) as u8 as f32;
        input[2] = game.can_win_on_next_turn((player_id + 2) % 3) as u8 as f32;
        input[3] = get_total_height(game, player_id) as f32;
        input[4] = get_total_height(game, (player_id + 1) % 3) as f32;
        input[5] = get_total_height(game, (player_id + 2) % 3) as f32;
		/*
        input[6] = get_max_height(game, player_id) as f32;
        input[7] = get_max_height(game, (player_id + 1) % 3) as f32;
        input[8] = get_max_height(game, (player_id + 2) % 3) as f32;
		input[9] = squares_movable_to(game, player_id) as f32;
        input[10] = squares_movable_to(game, (player_id + 1) % 3) as f32;
        input[11] = squares_movable_to(game, (player_id + 2) % 3) as f32;*/
		

        input
    }

    pub fn predict(&self, game: &Game, player_id: usize) -> f32 {
        self.network.predict(&self.generate_input(game, player_id))
    }

    pub fn learn(&mut self, games: &[(Game, usize, bool)], iterations: usize, step_size: f32) {
        let mut training_data: Vec<(f32, [f32; INPUT_COUNT])> = Vec::new();
        for (game, player_id, success) in games.iter() {
            training_data.push((
                if *success { 1.0 } else { -1.0 },
                self.generate_input(game, *player_id),
            ));
        }
        self.network.learn(&training_data, iterations, step_size);
    }
}
