use std::{
    collections::HashMap,
    fmt,
    sync::mpsc::{Receiver, Sender},
};

use egui::Color32;

use crate::{
    board::BoardData,
    board::{ConcreteBoard, Coordinate},
    render::{InteractionMessage, RenderMessage, RenderState},
    tile::{TileClickTarget, TileData},
    tilebag::TileBag,
};

pub struct RefereeState {
    pub tilebag: TileBag,
    pub board: ConcreteBoard,
    turn_order: Vec<Player>,
    turn_idx: usize,
    pub is_placing_meeple: bool,
    pub player_scores: HashMap<Player, u32>,
    placing_tile: Option<Coordinate>,
    pub player_meeples: HashMap<Player, u8>,
}

static INITIAL_MEEPLES: u8 = 7;

impl Default for RefereeState {
    fn default() -> Self {
        RefereeState {
            board: ConcreteBoard::default(),
            tilebag: TileBag::default(),
            turn_order: vec![Player::White, Player::Black],
            turn_idx: 0,
            is_placing_meeple: false,
            player_scores: HashMap::from([(Player::White, 0), (Player::Black, 0)]),
            player_meeples: HashMap::from([
                (Player::White, INITIAL_MEEPLES),
                (Player::Black, INITIAL_MEEPLES),
            ]),
            placing_tile: None,
        }
    }
}

impl RefereeState {
    pub fn clone_into(&self) -> RenderState {
        let player = self.get_player();
        RenderState {
            preview_tile: self.tilebag.peek().cloned(),
            board: self.board.clone(),
            is_placing_meeple: self.is_placing_meeple,
            turn_order: self.turn_order.clone(),
            current_player: player,
            player_scores: self.player_scores.clone(),
            player_meeples: self.player_meeples.clone(),
        }
    }
    fn score_placement(&mut self) {
        if let Some(coord) = &self.placing_tile {
            if let Some(tile) = self.board.at(coord) {
                let points = self.board.get_completion_points(coord, tile);
                for (maybe_player, addition) in points {
                    if let Some(player) = maybe_player {
                        if let Some(value) = self.player_scores.get_mut(&player) {
                            *value += addition as u32;
                        } else {
                            self.player_scores.insert(player, addition as u32);
                        }
                    }
                }
            }
        }
    }

    pub fn progress_phase(&mut self, placing_tile: Option<Coordinate>) {
        if self.is_placing_meeple {
            self.turn_idx = (self.turn_idx + 1) % self.turn_order.len();
            self.score_placement();

            self.is_placing_meeple = false;
            self.placing_tile = None;
        } else {
            self.is_placing_meeple = true;
            self.placing_tile = placing_tile;
        }
    }
    pub fn get_player(&self) -> Player {
        self.turn_order[self.turn_idx].clone()
    }
    pub fn is_legal_meeple_placement(&self, coord: Coordinate, target: &TileClickTarget) -> bool {
        self.board.is_legal_meeple(&coord, target.clone()).is_some()
    }
    pub fn is_legal_placement(&self, coord: Coordinate, tile: &TileData) -> bool {
        if self.board.tiles_placed() == 0 {
            return true;
        }
        let legal_tiles = self.board.get_legal_tiles();
        if !legal_tiles.contains(&coord) {
            return false;
        }

        if !self.board.is_features_match(&coord, tile) {
            return false;
        }
        true
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn get_color(&self) -> Color32 {
        match self {
            Self::Black => Color32::BLACK,
            Self::White => Color32::WHITE,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
            InteractionMessage::CancelMeeple => {
                if state.is_placing_meeple {
                    state.progress_phase(None)
                }
            }
            InteractionMessage::Click(message) => {
                if state.is_placing_meeple {
                    let player = state.get_player().clone();
                    let meeples_remaining = *state.player_meeples.get(&player).unwrap_or(&0);
                    if meeples_remaining > 0
                        && state.is_legal_meeple_placement(message.coord, &message.location)
                    {
                        let maybe_tile = state.board.at_mut(&message.coord);
                        if let Some(tile) = maybe_tile {
                            let success = tile.place_meeple(&message.location, &player);
                            if success {
                                state.player_meeples.insert(player, meeples_remaining - 1);
                                state.progress_phase(None);
                            }
                        }
                    }
                } else {
                    let maybe_next = state.tilebag.peek();

                    if let Some(next) = maybe_next {
                        let mut next = next.clone();
                        next.rotation = message.rotation.clone();
                        if state.is_legal_placement(message.coord, &next) {
                            state.tilebag.pull();

                            state.board.set(message.coord, next.clone());
                            state.progress_phase(Some(message.coord));
                        }
                    } else {
                        println!("out of tiles");
                    }
                }
            }
        }
        sender
            .send(RenderMessage::RefereeSync(state.clone_into()))
            .unwrap();
    }
}
