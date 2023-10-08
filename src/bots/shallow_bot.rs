use rayon::prelude::*;
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

        let own_player = self.get_own_player().clone();
        let tile = state.tilebag.peek().unwrap();
        let can_place = state
            .player_meeples
            .get(&own_player)
            .map(|ct| ct > &0)
            .unwrap_or(false);
        let moves: Vec<MoveRequest> = board_user.get_legal_moves(tile, can_place);
        let mut candidate: Option<(MoveRequest, i32)> = None;

        for move_request in moves {
            let total = (0..SAMPLING)
                .into_par_iter()
                .map(|_i| {
                    let mut out: i32 = 0;
                    let mut state = state.clone();
                    state.process_move(move_request.clone()).unwrap();
                    let result = Match::play_random_from_state(state).unwrap();
                    for (player, points) in result.player_scores {
                        if &player == &own_player {
                            out += points as i32;
                        } else {
                            out -= points as i32;
                        }
                    }
                    out
                })
                .sum();

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
