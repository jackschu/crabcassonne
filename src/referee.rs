use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::mpsc::{Receiver, Sender},
};

use egui::Color32;

use crate::{render::Message, tilebag::TileBag};

pub const BOARD_DIM: usize = 72;
const BOARD_SIZE: usize = BOARD_DIM * BOARD_DIM;

#[derive(Clone, Default)]
pub struct Board {
    data: HashMap<(usize, usize), TileData>,
}

impl Board {
    pub fn at(&self, row: usize, col: usize) -> Option<&TileData> {
        self.data.get(&(row, col))
    }
    pub fn at_mut(&mut self, row: usize, col: usize) -> Option<&mut TileData> {
        self.data.get_mut(&(row, col))
    }
    pub fn set(&mut self, row: usize, col: usize, tile: TileData) {
        self.data.insert((row, col), tile);
    }
    pub fn tiles_placed(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn get_legal_tiles(&self) -> HashSet<(usize, usize)> {
        let deltas: Vec<(i8, i8)> = vec![(0, 1), (1, 0), (-1, 0), (0, -1)];

        let output: HashSet<(usize, usize)> = self
            .data
            .keys()
            .flat_map(|tile| -> HashSet<(usize, usize)> {
                deltas
                    .iter()
                    .filter_map(|delta| {
                        let row = delta.0 + (tile.0 as i8);
                        if row < 0 || row > (BOARD_SIZE as i8) {
                            return None;
                        };
                        let col = delta.1 + (tile.1 as i8);
                        if col < 0 || col > (BOARD_SIZE as i8) {
                            return None;
                        };
                        if self.at(row as usize, col as usize).is_some() {
                            return None;
                        }
                        Some((row as usize, col as usize))
                    })
                    .collect()
            })
            .collect();
        output
    }
}

#[derive(Clone, Default)]
pub struct TileData {
    pub has_emblem: bool,
    pub top: MiniTile,
    pub left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    pub right: MiniTile,
    pub bottom: MiniTile,
}

impl TileData {
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
#[derive(Default)]
pub enum MiniTile {
    #[default]
    Grass,
    Road,
    City,
    Monastery,
    Junction,
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
                if board.tiles_placed() != 0 {
                    let legal_tiles = board.get_legal_tiles();
                    if !legal_tiles.contains(&(message.row, message.column)) {
                        continue;
                    }
                }
                if let Some(tile) = tilebag.pull() {
                    board.set(message.row, message.column, tile);
                } else {
                    println!("out of tiles");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn legal_tiles() {
        let mut board = Board::default();
        let mut bag = TileBag::default();
        board.set(30, 30, bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 4);
        board.set(29, 30, bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 6);
        board.set(29, 30, bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 6);

        board.set(29, 29, bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 7);

        board.set(30, 29, bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 8);
    }
}
