use std::collections::{HashMap, HashSet};

use crate::tile::TileData;

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

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::tilebag::TileBag;

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
