use std::sync::mpsc::{Receiver, Sender};

use egui::Color32;

use crate::{render::Message, tilebag::TileBag};

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

#[derive(Clone, Default)]
pub struct PlacedTile {
    pub has_emblem: bool,
    pub top: MiniTile,
    pub left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    pub right: MiniTile,
    pub bottom: MiniTile,
}

impl PlacedTile {
    pub fn at(&self, target: &TileClickTarget) -> &MiniTile {
        match target {
            TileClickTarget::Top => &self.top,
            TileClickTarget::Left => &self.left,
            TileClickTarget::Center => &self.center,
            TileClickTarget::Right => &self.right,
            TileClickTarget::Bottom => &self.bottom,
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
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

impl Default for MiniTile {
    fn default() -> Self {
        MiniTile::Grass
    }
}

impl MiniTile {
    pub fn get_color(&self) -> Color32 {
        match self {
            Self::Grass => Color32::from_rgb(0, 188, 84),
            Self::Road => Color32::WHITE,
            Self::City => Color32::from_rgb(205, 137, 48),
            Self::Monastery => Color32::RED,
            Self::Junction => Color32::YELLOW,
        }
    }
}

pub fn referee_main(receiver: Receiver<Message>, sender: Sender<Board>) {
    let mut board = Board::default();
    let mut tilebag = TileBag::default();
    loop {
        sender.send(board.clone()).unwrap();
        match receiver.recv().unwrap() {
            Message::Print(message) => {
                println!("recv {}", message);
            }
            Message::Click(message) => {
                board.set(message.row, message.column, Some(tilebag.pull()));
            }
        }
    }
}
