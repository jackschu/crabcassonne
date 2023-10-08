use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    board::BoardData,
    referee::{Player, RefereeState},
};

use super::bot::{Bot, MoveRequest};

pub struct RandomBot {
    pub own_player: Player,
    rng: ThreadRng,
}

impl RandomBot {
    pub fn new(player: Player) -> Self {
        RandomBot {
            own_player: player,
            rng: rand::thread_rng(),
        }
    }
}

impl Bot for RandomBot {
    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_name(&self) -> String {
        "random bot".to_owned()
    }

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let board = &state.board;

        let tile = state.tilebag.peek().unwrap();
        let can_place = state
            .player_meeples
            .get(self.get_own_player())
            .map(|ct| ct > &0)
            .unwrap_or(false);
        let out: Vec<MoveRequest> = board.as_user().get_legal_moves(tile, can_place);

        let idx = self.rng.gen_range(0..out.len());
        out[idx].clone()
    }
}
