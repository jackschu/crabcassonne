use std::sync::mpsc::{Receiver, Sender};

use egui::Color32;

use crate::render::Message;

const BOARD_DIM: usize = 72;
const BOARD_SIZE: usize = BOARD_DIM * BOARD_DIM;

#[derive(Clone)]
pub struct Board {
    data: Vec<Option<PlacedTile>>,
}

impl Board {
    pub fn at(&self, row: usize, col: usize) -> &Option<PlacedTile> {
        &self.data[BOARD_DIM * col + row]
    }
    pub fn at_mut(&mut self, row: usize, col: usize) -> &mut Option<PlacedTile> {
        &mut self.data[BOARD_DIM * col + row]
    }
    pub fn set(&mut self, row: usize, col: usize, tile: Option<PlacedTile>) {
        self.data[BOARD_DIM * col + row] = tile
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            data: vec![None; BOARD_SIZE],
        }
    }
}

#[derive(Clone)]
pub struct PlacedTile {
    pub has_emblem: bool,
    // index using mini tile
    data: [MiniTile; 5],
}

impl PlacedTile {
    pub fn at(&self, target: &TileClickTarget) -> &MiniTile {
        let idx = match target {
            TileClickTarget::Top => 0,
            TileClickTarget::Left => 1,
            TileClickTarget::Center => 2,
            TileClickTarget::Right => 3,
            TileClickTarget::Bottom => 4,
        };
        &self.data[idx]
    }

    pub fn create_grass() -> Self {
        PlacedTile {
            has_emblem: false,
            data: [
                MiniTile::Grass,
                MiniTile::Grass,
                MiniTile::Grass,
                MiniTile::Grass,
                MiniTile::Grass,
            ],
        }
    }
}

#[derive(Clone, Hash)]
pub enum TileClickTarget {
    Top,
    Left,
    Center,
    Right,
    Bottom,
}

#[derive(Clone)]
pub enum MiniTile {
    Grass,
    Road,
    City,
    Monastery,
    Junction,
}

impl MiniTile {
    pub fn get_color(&self) -> Color32 {
        match self {
            Self::Grass => Color32::GREEN,
            Self::Road => Color32::WHITE,
            Self::City => Color32::BROWN,
            Self::Monastery => Color32::RED,
            Self::Junction => Color32::YELLOW,
        }
    }
}

pub fn referee_main(receiver: Receiver<Message>, sender: Sender<Board>) {
    let mut board = Board::default();
    board.data[0] = Some(PlacedTile::create_grass());
    loop {
        sender.send(board.clone()).unwrap();
        match receiver.recv().unwrap() {
            Message::Print(message) => {
                println!("recv {}", message);
            }
            Message::Click(message) => {
                board.set(
                    message.row,
                    message.column,
                    Some(PlacedTile::create_grass()),
                );
            }
        }
    }
}
