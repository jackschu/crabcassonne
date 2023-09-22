use std::collections::HashMap;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    board::{BoardData, ConcreteBoard, Coordinate},
    referee::Player,
    tile::{Rotation, TileClickTarget, TileData},
};

pub trait Bot {
    fn get_own_player(&self) -> &Player;
    fn get_move(
        &mut self,
        tile: &TileData,
        board: &ConcreteBoard,
        scores: HashMap<Player, u32>,
        meeples: HashMap<Player, u8>,
    ) -> MoveRequest;
}

pub struct RandomBot {
    pub own_player: Player,
    pub rng: ThreadRng,
}

#[derive(Clone)]
pub struct MoveRequest {
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub meeple: Option<TileClickTarget>,
}

impl Bot for RandomBot {
    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_move(
        &mut self,
        tile: &TileData,
        board: &ConcreteBoard,
        scores: HashMap<Player, u32>,
        meeples: HashMap<Player, u8>,
    ) -> MoveRequest {
        let coords = board.get_legal_tiles();
        let meeple_dests = [
            TileClickTarget::Top,
            TileClickTarget::Bottom,
            TileClickTarget::Left,
            TileClickTarget::Right,
            TileClickTarget::Center,
        ];
        let mut out: Vec<MoveRequest> = vec![];
        for coord in coords {
            for rotation in [
                Rotation::None,
                Rotation::Left,
                Rotation::Flip,
                Rotation::Right,
            ] {
                let mut tile = tile.clone();
                tile.rotation = rotation.clone();
                if board.is_features_match(&coord, &tile) {
                    let board = board.with_overlay(coord.clone(), &tile);
                    if meeples
                        .get(&self.get_own_player())
                        .map(|ct| ct > &0)
                        .unwrap_or(false)
                    {
                        for dest in &meeple_dests {
                            if board.is_legal_meeple(&coord, dest.clone()).is_some() {
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
            }
        }
        let idx = self.rng.gen_range(0..out.len());
        out[idx].clone()
    }
}
