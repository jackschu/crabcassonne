use std::collections::VecDeque;

use crate::referee::{Player, RefereeState};

use super::bot::{Bot, MoveRequest};

pub struct ReplayBot {
    pub own_player: Player,
    pub moves: VecDeque<MoveRequest>,
}

impl ReplayBot {
    pub fn unitialized(player: Player) -> Self {
        ReplayBot {
            own_player: player,
            moves: VecDeque::new(),
        }
    }
    pub fn add_move(&mut self, the_move: MoveRequest) {
        self.moves.push_back(the_move);
    }
}

impl Bot for ReplayBot {
    fn get_own_player(&self) -> &Player {
        &self.own_player
    }
    fn get_move(&mut self, _state: &RefereeState) -> MoveRequest {
        self.moves.pop_front().unwrap()
    }
}
