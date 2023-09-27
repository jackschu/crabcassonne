use std::cmp::Ordering;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    board::BoardData,
    referee::{Player, RefereeState},
};

use super::bot::{Bot, MoveRequest};

pub struct GreedyBot {
    pub own_player: Player,
    rng: ThreadRng,
}

impl GreedyBot {
    pub fn new(player: Player) -> Self {
        GreedyBot {
            own_player: player,
            rng: rand::thread_rng(),
        }
    }
}

impl Bot for GreedyBot {
    fn get_name(&self) -> &str {
        "greedy bot"
    }

    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let board_user = state.board.as_user();

        let tile = state.tilebag.peek().unwrap();
        let can_place = state
            .player_meeples
            .get(self.get_own_player())
            .map(|ct| ct > &0)
            .unwrap_or(false);
        let moves: Vec<MoveRequest> = board_user.get_legal_moves(tile, can_place);
        let mut candidate: Option<(MoveRequest, i32)> = None;
        for move_request in moves {
            let mut tile = tile.clone();
            tile.rotation = move_request.rotation.clone();
            let points = board_user.get_completion_points(&move_request.coord, &tile);
            let mut total: i32 = 0;
            for (player, points) in points {
                if let Some(player) = player {
                    if &player == self.get_own_player() {
                        total += points as i32;
                    } else {
                        total -= points as i32;
                    }
                }
            }
            if let Some((_request, score)) = candidate.clone() {
                match score.cmp(&total) {
                    Ordering::Less => {
                        candidate = Some((move_request.clone(), total));
                    }
                    Ordering::Equal => {
                        if self.rng.gen_bool(0.5) {
                            candidate = Some((move_request.clone(), total));
                        }
                    }
                    Ordering::Greater => {}
                }
            } else {
                candidate = Some((move_request.clone(), total));
            }
        }
        candidate.unwrap().0
    }
}
