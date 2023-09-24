use std::{
    rc::Rc,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{
    arena::MessageResult,
    board::{BoardData, Coordinate},
    referee::{Player, RefereeState},
    render::{InteractionMessage, RenderMessage},
    tile::{Rotation, TileClickTarget},
};

pub trait Bot {
    fn get_own_player(&self) -> &Player;
    fn get_move(&mut self, state: &RefereeState) -> MoveRequest;
}

pub struct RandomBot {
    pub own_player: Player,
    rng: ThreadRng,
}

pub struct HumanBot {
    pub own_player: Player,
    receiver: Rc<Mutex<Receiver<InteractionMessage>>>,
    sender: Sender<RenderMessage>,
}

impl HumanBot {
    pub fn new(
        player: Player,
        receiver: Rc<Mutex<Receiver<InteractionMessage>>>,
        sender: Sender<RenderMessage>,
    ) -> Self {
        Self {
            own_player: player,
            receiver,
            sender,
        }
    }

    pub fn validate_tile_placement(
        &self,
        coord: Coordinate,
        rotation: Rotation,
        state: &RefereeState,
    ) -> MessageResult<()> {
        let next = state.tilebag.peek()?;

        let mut next = next.clone();
        next.rotation = rotation;
        state.is_legal_placement(coord, &next)
    }

    pub fn validate_meeple_placement(
        &self,
        coord: Coordinate,
        rotation: Rotation,
        location: TileClickTarget,
        state: &RefereeState,
    ) -> MessageResult<()> {
        let next = state.tilebag.peek()?;

        let mut next = next.clone();
        next.rotation = rotation;
        let board = state.board.with_overlay(coord, &next);
        board.as_user().is_legal_meeple(&coord, location)
    }
}

impl Bot for HumanBot {
    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let mut is_placing_meeple = false;
        let mut tile_data: Option<(Coordinate, Rotation)> = None;
        loop {
            self.sender
                .send(RenderMessage::RefereeSync(
                    state.clone_into_mid_move(tile_data.clone(), is_placing_meeple),
                ))
                .unwrap();
            match self.receiver.lock().unwrap().recv().unwrap() {
                InteractionMessage::Print(message) => {
                    println!("recv {}", message);
                }
                InteractionMessage::CancelMeeple => {
                    if is_placing_meeple {
                        if let Some((coord, rotation)) = tile_data {
                            return MoveRequest {
                                coord,
                                rotation,
                                meeple: None,
                            };
                        }
                    }
                }
                InteractionMessage::Click(message) => {
                    if is_placing_meeple {
                        if let Some((coord, rotation)) = &tile_data {
                            let attempt = self.validate_meeple_placement(
                                message.coord,
                                rotation.clone(),
                                message.location.clone(),
                                state,
                            );
                            if attempt.is_ok() {
                                return MoveRequest {
                                    coord: *coord,
                                    rotation: rotation.clone(),
                                    meeple: Some(message.location),
                                };
                            } else {
                                println!("{:?}", attempt);
                            }
                        }
                    } else {
                        let attempt = self.validate_tile_placement(
                            message.coord,
                            message.rotation.clone(),
                            state,
                        );
                        if attempt.is_ok() {
                            tile_data = Some((message.coord, message.rotation.clone()));
                            is_placing_meeple = true;
                        } else {
                            println!("{:?}", attempt);
                        };
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MoveRequest {
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub meeple: Option<TileClickTarget>,
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
