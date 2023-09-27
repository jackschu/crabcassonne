use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    board::BoardData,
    referee::{Player, RefereeState},
    tile::TileClickTarget,
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

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let board = &state.board;

        let mut coords = board.as_user().get_legal_tiles();
        if board.tiles_present().count() == 0 {
            coords.insert((0, 0));
        }
        let tile = state.tilebag.peek().unwrap();
        let meeples = &state.player_meeples;
        let meeple_dests = [
            TileClickTarget::Top,
            TileClickTarget::Bottom,
            TileClickTarget::Left,
            TileClickTarget::Right,
            TileClickTarget::Center,
        ];
        let mut out: Vec<MoveRequest> = vec![];
        for (coord, rotation) in board.as_user().get_legal_moves(tile) {
            let mut tile = tile.clone();
            tile.rotation = rotation.clone();
            let board = board.with_overlay(coord, &tile);
            if meeples
                .get(self.get_own_player())
                .map(|ct| ct > &0)
                .unwrap_or(false)
            {
                for dest in &meeple_dests {
                    if board
                        .as_user()
                        .is_legal_meeple(&coord, dest.clone())
                        .is_ok()
                    {
                        out.push(MoveRequest {
                            coord,
                            rotation: rotation.clone(),
                            meeple: Some(dest.clone()),
                        })
                    }
                }
            }
            out.push(MoveRequest {
                coord,
                rotation: rotation.clone(),
                meeple: None,
            })
        }
        let idx = self.rng.gen_range(0..out.len());
        out[idx].clone()
    }
}
