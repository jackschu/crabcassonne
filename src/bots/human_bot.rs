use crate::tilebag::TileBag;
use std::{
    rc::Rc,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

use crate::{
    arena::MessageResult,
    board::{BoardData, Coordinate},
    referee::{Player, RefereeState},
    render::{InteractionMessage, RenderMessage},
    tile::{Rotation, TileClickTarget},
};

use super::bot::{Bot, MoveRequest};
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
        let player = self.get_own_player();
        let count = state.player_meeples.get(player).unwrap_or(&0);
        if *count <= 0 {
            return Err("No meeples remaining");
        }
        let next = state.tilebag.peek()?;

        let mut next = next.clone();
        next.rotation = rotation;
        let board = state.board.with_overlay(coord, &next);
        board.as_user().is_legal_meeple(&coord, location)
    }
}

impl Bot for HumanBot {
    fn get_name(&self) -> String {
        "human".to_owned()
    }
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
                InteractionMessage::NextFrame
                | InteractionMessage::PreviousFrame
                | InteractionMessage::FirstFrame
                | InteractionMessage::LastFrame => {}
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
