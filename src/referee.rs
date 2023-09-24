use std::{
    collections::HashMap,
    fmt,
    sync::mpsc::{Receiver, Sender},
};

use egui::Color32;

use crate::{
    arena::MessageResult,
    board::BoardData,
    board::{BoardUser, ConcreteBoard, Coordinate},
    bot::MoveRequest,
    render::{InteractionMessage, RenderMessage, RenderState},
    tile::{Rotation, TileClickTarget, TileData},
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
    pub fn process_move(&mut self, move_request: MoveRequest) -> MessageResult<()> {
        self.handle_tile_placement(move_request.coord, move_request.rotation)?;
        if let Some(location) = move_request.meeple {
            self.handle_meeple_placement(move_request.coord, location)
        } else {
            self.progress_phase(None);
            Ok(())
        }
    }

    pub fn from_players(players: Vec<Player>) -> Self {
        let player_scores: HashMap<Player, u32> = players.iter().map(|p| (p.clone(), 0)).collect();
        let player_meeples: HashMap<Player, u8> = players
            .iter()
            .map(|p| (p.clone(), INITIAL_MEEPLES))
            .collect();
        RefereeState {
            turn_order: players,
            player_scores,
            player_meeples,
            ..Default::default()
        }
    }
    pub fn clone_into_mid_move(
        &self,
        preview_placed: Option<(Coordinate, Rotation)>,
        is_placing_meeple: bool,
    ) -> RenderState {
        let player = self.get_player();
        let mut board = self.board.clone();
        let preview_tile = self.tilebag.peek().ok().cloned();
        if let Some((coord, rotation)) = preview_placed {
            if let Some(tile) = &preview_tile {
                let mut tile = tile.clone();
                tile.rotation = rotation;
                board.set(coord, tile);
            }
        }
        RenderState {
            preview_tile,
            board,
            is_placing_meeple,
            turn_order: self.turn_order.clone(),
            current_player: player,
            player_scores: self.player_scores.clone(),
            player_meeples: self.player_meeples.clone(),
        }
    }

    pub fn clone_into(&self) -> RenderState {
        let player = self.get_player();
        RenderState {
            preview_tile: self.tilebag.peek().ok().cloned(),
            board: self.board.clone(),
            is_placing_meeple: self.is_placing_meeple,
            turn_order: self.turn_order.clone(),
            current_player: player,
            player_scores: self.player_scores.clone(),
            player_meeples: self.player_meeples.clone(),
        }
    }
    fn board_user(&self) -> BoardUser {
        BoardUser {
            board: Box::new(&self.board),
        }
    }
    fn score_placement(&mut self) {
        if let Some(coord) = &self.placing_tile {
            if let Some(tile) = self.board.at(coord) {
                let score_data = self.board_user().get_feature_score_data(coord, tile);
                let points = self.board_user().get_points_from_score_data(&score_data);
                for (maybe_player, addition) in points {
                    if let Some(player) = maybe_player {
                        if let Some(value) = self.player_scores.get_mut(&player) {
                            *value += addition as u32;
                        } else {
                            self.player_scores.insert(player, addition as u32);
                        }
                    }
                }
                for score in score_data {
                    if !score.completed {
                        continue;
                    }
                    let visited = score.removal_candidate;
                    let removed = self.board.remove_meeples(visited);
                    for (player, count) in removed {
                        if let Some(stored_count) = self.player_meeples.get_mut(&player) {
                            *stored_count += count;
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

    pub fn handle_tile_placement(
        &mut self,
        coord: Coordinate,
        rotation: Rotation,
    ) -> MessageResult<()> {
        let next = self.tilebag.peek()?;

        let mut next = next.clone();
        next.rotation = rotation;
        self.is_legal_placement(coord, &next)?;
        self.tilebag.pull();

        self.board.set(coord, next);
        self.progress_phase(Some(coord));
        Ok(())
    }

    pub fn handle_meeple_placement(
        &mut self,
        coord: Coordinate,
        location: TileClickTarget,
    ) -> MessageResult<()> {
        let player = self.get_player();
        let meeples_remaining = *self.player_meeples.get(&player).unwrap_or(&0);
        if meeples_remaining == 0 {
            return Err("out of meeples");
        }
        if !self.is_legal_meeple_placement(coord, &location) {
            return Err("illegal meeple placement");
        }
        let tile = self.board.at_mut(&coord);
        if let Some(tile) = tile {
            tile.place_meeple(&location, &player)?;
            self.player_meeples.insert(player, meeples_remaining - 1);
            self.progress_phase(None);
            Ok(())
        } else {
            Err("placing meeple on non existant tile")
        }
    }
    pub fn get_player(&self) -> Player {
        self.turn_order[self.turn_idx].clone()
    }
    pub fn is_legal_meeple_placement(&self, coord: Coordinate, target: &TileClickTarget) -> bool {
        self.board_user()
            .is_legal_meeple(&coord, target.clone())
            .is_ok()
    }
    pub fn is_legal_placement(&self, coord: Coordinate, tile: &TileData) -> MessageResult<()> {
        if self.board_user().tiles_placed() == 0 {
            return Ok(());
        }
        let legal_tiles = self.board_user().get_legal_tiles();
        if !legal_tiles.contains(&coord) {
            return Err("Illegal tile placement: No connecting tile");
        }

        if !self.board_user().is_features_match(&coord, tile) {
            return Err("Illegal tile placement: features dont match");
        }
        Ok(())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Player {
    White,
    Black,
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
                    if state
                        .handle_meeple_placement(message.coord, message.location)
                        .is_err()
                    {
                        println!("illegal meeple placement");
                    }
                } else if state
                    .handle_tile_placement(message.coord, message.rotation)
                    .is_err()
                {
                    println!("illegal tile placement");
                }
            }
        }
        sender
            .send(RenderMessage::RefereeSync(state.clone_into()))
            .unwrap();
    }
}
