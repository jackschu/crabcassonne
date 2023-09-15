use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    board::BoardData,
    board::{ConcreteBoard, Coordinate},
    render::{InteractionMessage, RenderMessage, RenderState},
    tile::TileData,
    tilebag::TileBag,
};

pub struct RefereeState {
    pub tilebag: TileBag,
    pub board: ConcreteBoard,
    turn_order: Vec<Player>,
    turn_idx: usize,
    player_scores: HashMap<Player, u32>,
}

impl Default for RefereeState {
    fn default() -> Self {
        RefereeState {
            board: ConcreteBoard::default(),
            tilebag: TileBag::default(),
            turn_order: vec![Player::White, Player::Black],
            turn_idx: 0,
            player_scores: HashMap::from([(Player::White, 0), (Player::Black, 0)]),
        }
    }
}

impl RefereeState {
    pub fn clone_into(&self) -> RenderState {
        let player = self.get_player();
        RenderState {
            preview_tile: self.tilebag.peek().map(|x| x.clone()),
            board: self.board.clone(),
            turn_order: self.turn_order.clone(),
            current_player: player,
            player_scores: self.player_scores.clone(),
        }
    }
    pub fn progress_turn(&mut self) {
        self.turn_idx = (self.turn_idx + 1) % self.turn_order.len();
    }
    pub fn get_player(&self) -> Player {
        self.turn_order[self.turn_idx].clone()
    }
    pub fn is_legal_placement(&self, coord: Coordinate, tile: &TileData) -> bool {
        if self.board.tiles_placed() == 0 {
            return true;
        }
        let legal_tiles = self.board.get_legal_tiles();
        if !legal_tiles.contains(&coord) {
            return false;
        }

        if !self.board.is_features_match(&coord, &tile) {
            return false;
        }
        return true;
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Player {
    Black,
    White,
}

pub fn referee_main(receiver: Receiver<InteractionMessage>, sender: Sender<RenderMessage>) {
    let mut state = RefereeState::default();
    sender
        .send(RenderMessage::RefereeSync(state.clone_into()))
        .unwrap();
    loop {
        match receiver.recv().unwrap() {
            InteractionMessage::Print(message) => {
                println!("recv {}", message);
            }
            InteractionMessage::Click(message) => {
                let maybe_next = state.tilebag.peek();

                if let Some(next) = maybe_next {
                    let mut next = next.clone();
                    next.rotation = message.rotation.clone();
                    if state.is_legal_placement(message.coord, &next) {
                        state.tilebag.pull();
                        let points = state.board.get_completion_points(&message.coord, &next);
                        println!("scored points {}", points);
                        state.board.set(message.coord, next);
                        state.progress_turn();
                    }
                } else {
                    println!("out of tiles");
                }
            }
        }
        sender
            .send(RenderMessage::RefereeSync(state.clone_into()))
            .unwrap();
    }
}
