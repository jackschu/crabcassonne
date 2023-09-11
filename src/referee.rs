use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::mpsc::{Receiver, Sender},
};

use egui::Color32;

use crate::{
    render::{InteractionMessage, RenderMessage},
    tilebag::TileBag,
};

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
pub struct TileDataBuilder {
    pub has_emblem: bool,
    pub top: MiniTile,
    pub left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    pub right: MiniTile,
    pub bottom: MiniTile,
}

#[derive(Clone, Debug)]
pub struct TileData {
    pub has_emblem: bool,
    top: MiniTile,
    left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    right: MiniTile,
    bottom: MiniTile,
    pub rotation: Rotation,
}

#[derive(Clone, Debug)]
pub enum Rotation {
    None,
    Left,
    Right,
    Flip,
}

impl Rotation {
    pub fn next_right(&self) -> Rotation {
        match &self {
            Rotation::None => Rotation::Right,
            Rotation::Right => Rotation::Flip,
            Rotation::Flip => Rotation::Left,
            Rotation::Left => Rotation::None,
        }
    }
    pub fn next_left(&self) -> Rotation {
        match &self {
            Rotation::None => Rotation::Left,
            Rotation::Left => Rotation::Flip,
            Rotation::Flip => Rotation::Right,
            Rotation::Right => Rotation::None,
        }
    }
}

impl TileData {
    pub fn top(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.top,
            Rotation::Left => &self.right,
            Rotation::Right => &self.left,
            Rotation::Flip => &self.bottom,
        }
    }
    pub fn bottom(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.bottom,
            Rotation::Left => &self.left,
            Rotation::Right => &self.right,
            Rotation::Flip => &self.top,
        }
    }

    pub fn right(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.right,
            Rotation::Left => &self.bottom,
            Rotation::Right => &self.top,
            Rotation::Flip => &self.left,
        }
    }
    pub fn left(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.left,
            Rotation::Left => &self.top,
            Rotation::Right => &self.bottom,
            Rotation::Flip => &self.right,
        }
    }
    pub fn rotate_right(&mut self) {
        self.rotation = self.rotation.next_right();
    }
    pub fn rotate_left(&mut self) {
        self.rotation = self.rotation.next_left();
    }
}

impl From<TileDataBuilder> for TileData {
    fn from(builder: TileDataBuilder) -> TileData {
        TileData {
            has_emblem: builder.has_emblem,
            top: builder.top,
            left: builder.left,
            center: builder.center,
            secondary_center: builder.secondary_center,
            right: builder.right,
            bottom: builder.bottom,
            rotation: Rotation::None,
        }
    }
}

impl TileData {
    pub fn at(&self, target: &TileClickTarget) -> &MiniTile {
        match target {
            TileClickTarget::Top => &self.top(),
            TileClickTarget::Left => &self.left(),
            TileClickTarget::Center => &self.center,
            TileClickTarget::Right => &self.right(),
            TileClickTarget::Bottom => &self.bottom(),
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

pub fn referee_main(receiver: Receiver<InteractionMessage>, sender: Sender<RenderMessage>) {
    let mut board = Board::default();
    let mut tilebag = TileBag::default();
    if let Some(tile) = tilebag.peek() {
        sender
            .send(RenderMessage::PreviewTile(tile.clone()))
            .unwrap();
    }
    loop {
        sender.send(RenderMessage::NewBoard(board.clone())).unwrap();
        match receiver.recv().unwrap() {
            InteractionMessage::Print(message) => {
                println!("recv {}", message);
            }
            InteractionMessage::Click(message) => {
                if board.tiles_placed() != 0 {
                    let legal_tiles = board.get_legal_tiles();
                    if !legal_tiles.contains(&(message.row, message.column)) {
                        continue;
                    }
                }
                if let Some(mut tile) = tilebag.pull() {
                    tile.rotation = message.rotation;
                    board.set(message.row, message.column, tile);
                } else {
                    println!("out of tiles");
                }
                if let Some(tile) = tilebag.peek() {
                    sender
                        .send(RenderMessage::PreviewTile(tile.clone()))
                        .unwrap();
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

    #[test]
    fn tile_rotation() {
        let mut tile: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::Road,
            right: MiniTile::Junction,
            bottom: MiniTile::Monastery,
            ..Default::default()
        }
        .into();
        let original = tile.clone();

        tile.rotate_left();
        tile.rotate_right();

        for _ in 0..4 {
            tile.rotate_right();
        }

        for _ in 0..4 {
            tile.rotate_left();
        }

        assert_eq!(original.top(), tile.top());
        assert_eq!(original.bottom(), tile.bottom());
        assert_eq!(original.left(), tile.left());
        assert_eq!(original.right(), tile.right());

        tile.rotate_right();

        assert_eq!(original.left(), tile.top());
        assert_eq!(original.right(), tile.bottom());
        assert_eq!(original.top(), tile.right());
        assert_eq!(original.bottom(), tile.left());

        tile.rotate_left();
        tile.rotate_left();

        assert_eq!(original.right(), tile.top());
        assert_eq!(original.left(), tile.bottom());
        assert_eq!(original.top(), tile.left());
        assert_eq!(original.bottom(), tile.right());
    }
}
