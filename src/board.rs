use std::collections::{HashMap, HashSet};

use crate::tile::TileData;

pub const BOARD_DIM: usize = 72;

pub type Coordinate = (i8, i8);

#[derive(Clone, Default)]
pub struct Board {
    data: HashMap<Coordinate, TileData>,
}

impl Board {
    pub fn at(&self, coord: &Coordinate) -> Option<&TileData> {
        self.data.get(coord)
    }
    pub fn at_mut(&mut self, coord: &Coordinate) -> Option<&mut TileData> {
        self.data.get_mut(coord)
    }
    pub fn set(&mut self, coord: Coordinate, tile: TileData) {
        self.data.insert(coord, tile);
    }
    pub fn tiles_placed(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn get_legal_tiles(&self) -> HashSet<Coordinate> {
        let deltas: Vec<Coordinate> = vec![(0, 1), (1, 0), (-1, 0), (0, -1)];

        self.data
            .keys()
            .flat_map(|tile| -> HashSet<Coordinate> {
                deltas
                    .iter()
                    .filter_map(|delta| {
                        let coord = (delta.0 + tile.0, delta.1 + tile.1);
                        if self.data.contains_key(&coord) {
                            None
                        } else {
                            Some(coord)
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::tilebag::TileBag;

    use super::*;

    #[test]
    fn legal_tiles() {
        let mut board = Board::default();
        let mut bag = TileBag::default();
        board.set((30, 30), bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 4);
        board.set((29, 30), bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 6);
        board.set((29, 30), bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 6);

        board.set((29, 29), bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 7);

        board.set((30, 29), bag.pull().unwrap());
        assert_eq!(board.get_legal_tiles().len(), 8);
    }
}
