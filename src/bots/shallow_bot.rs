use std::cmp::Ordering;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    arena::Match,
    board::BoardData,
    referee::{Player, RefereeState},
};

use super::bot::{Bot, MoveRequest};

pub struct ShallowBot {
    pub own_player: Player,
    rng: ThreadRng,
}

impl ShallowBot {
    pub fn new(player: Player) -> Self {
        ShallowBot {
            own_player: player,
            rng: rand::thread_rng(),
        }
    }
}

const SAMPLING: u8 = 10;

impl Bot for ShallowBot {
    fn get_name(&self) -> &str {
        "shallow bot"
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
            let mut total: i32 = 0;

            for _i in 0..SAMPLING {
                let mut state = state.clone();

                state.process_move(move_request.clone()).unwrap();
                let result = Match::play_random_from_state(state).unwrap();
                for (player, points) in result.player_scores {
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
