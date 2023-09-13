use std::collections::{HashMap, HashSet};

use crate::tile::{MiniTile, TileClickTarget, TileData, CARDINALS};
use once_cell::sync::Lazy;

pub const BOARD_DIM: usize = 72;

pub type Coordinate = (i8, i8);

#[derive(Clone, Default)]
pub struct Board {
    data: HashMap<Coordinate, TileData>,
}

static DELTAS: [Coordinate; 4] = [(0, 1), (1, 0), (-1, 0), (0, -1)];
static DELTAS_MAP: Lazy<HashMap<TileClickTarget, Coordinate>> = Lazy::new(|| {
    HashMap::from([
        (TileClickTarget::Right, (0, 1)),
        (TileClickTarget::Bottom, (1, 0)),
        (TileClickTarget::Top, (-1, 0)),
        (TileClickTarget::Left, (0, -1)),
    ])
});

// From unplaced tile to placed tile
static COUPLINGS: [(TileClickTarget, TileClickTarget); 4] = [
    (TileClickTarget::Right, TileClickTarget::Left),
    (TileClickTarget::Bottom, TileClickTarget::Top),
    (TileClickTarget::Top, TileClickTarget::Bottom),
    (TileClickTarget::Left, TileClickTarget::Right),
];

static COUPLINGS_MAP: Lazy<HashMap<TileClickTarget, TileClickTarget>> =
    Lazy::new(|| HashMap::from(COUPLINGS.clone()));

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

    pub fn get_completion_points(&self, tile: &TileData, coord: &Coordinate) -> u8 {
        for direction in &CARDINALS {
            let feature = tile.at(&direction);
            let (included, completed) = self.get_feature_tiles(tile, coord, &direction);
            let non_empty: HashSet<Coordinate> = included
                .into_iter()
                .map(|x| x.0)
                .filter(|x| self.at(&x).is_some())
                .collect();

            println!(
                "feature {:?} len {} completed {}",
                feature,
                non_empty.len() + 1,
                completed
            );
        }
        0 // TODO
    }

    fn offset_coordinate(coord: &Coordinate, direction: &TileClickTarget) -> Coordinate {
        let offset = DELTAS_MAP.get(direction).unwrap();
        (coord.0 + offset.0, coord.1 + offset.1)
    }
    pub fn get_feature_tiles(
        &self,
        initial_tile: &TileData,
        initial_coord: &Coordinate,
        direction: &TileClickTarget,
    ) -> (Vec<(Coordinate, TileClickTarget)>, bool) {
        #[allow(clippy::single_match)] // will expand
        let feature = initial_tile.at(&direction);
        match feature {
            MiniTile::Road | MiniTile::City => {
                let mut complete = true;
                let mut queue = vec![(initial_coord.clone(), direction.clone())];
                let mut visited = HashSet::from([(initial_coord.clone(), direction.clone())]);

                while let Some((coord, direction)) = queue.pop() {
                    let maybe_tile = if coord == *initial_coord {
                        Some(initial_tile)
                    } else {
                        self.at(&coord)
                    };
                    match maybe_tile {
                        None => complete = false,
                        Some(tile) => {
                            let directions = tile.get_exits(&direction);
                            let next: Vec<(Coordinate, TileClickTarget)> = directions
                                .iter()
                                .map(|direction| {
                                    (
                                        Board::offset_coordinate(&coord, direction),
                                        COUPLINGS_MAP.get(direction).unwrap().clone(),
                                    )
                                })
                                .filter(|elem| visited.get(&elem).is_none())
                                .collect();
                            for elem in next {
                                visited.insert(elem.clone());
                                queue.push(elem);
                            }
                        }
                    }
                }
                (visited.into_iter().collect(), complete)
            }
            _ => (vec![], false),
        }
    }

    pub fn get_legal_tiles(&self) -> HashSet<Coordinate> {
        self.data
            .keys()
            .flat_map(|tile| -> HashSet<Coordinate> {
                DELTAS
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

    pub fn is_features_match(&self, dest: &Coordinate, incoming_tile: &TileData) -> bool {
        DELTAS
            .iter()
            .map(|delta| self.at(&(delta.0 + dest.0, delta.1 + dest.1)))
            .zip(COUPLINGS.iter())
            .filter_map(|(existing_tile, coupling)| {
                let tile = existing_tile?;
                Some((tile, coupling))
            })
            .all(|(existing_tile, coupling)| {
                let incoming_loc = &coupling.0;
                let existing_loc = &coupling.1;

                existing_tile.at(existing_loc) == incoming_tile.at(incoming_loc)
            })
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::{
        tile::{MiniTile, TileDataBuilder},
        tilebag::TileBag,
    };

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
    #[test]
    fn features_match_basic() {
        let mut board = Board::default();
        let tile_left: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::Road,
            right: MiniTile::Junction,
            bottom: MiniTile::Monastery,
            ..Default::default()
        }
        .into();
        let mut tile_right = tile_left.clone();
        board.set((30, 30), tile_left);
        assert!(!board.is_features_match(&(30, 31), &tile_right));

        tile_right.rotate_right();
        assert!(!board.is_features_match(&(30, 31), &tile_right));
        tile_right.rotate_right();

        assert!(board.is_features_match(&(30, 31), &tile_right));
    }

    // #[test]
    // fn feature_completion_test() {
    //     let mut board = Board::default();
    //     let tile_city: TileData = TileDataBuilder {
    //         top: MiniTile::City,
    //         left: MiniTile::City,
    //         right: MiniTile::City,
    //         bottom: MiniTile::City,
    //         ..Default::default()
    //     }
    //     .into();

    //     board.get_completion_points(&tile_city, &(30, 30));
    //     assert!(false);
    // }

    #[test]
    fn features_match_surround_city() {
        let mut board = Board::default();

        let tile_city: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::City,
            right: MiniTile::City,
            bottom: MiniTile::City,
            ..Default::default()
        }
        .into();

        let center = (30 as i8, 30 as i8);
        for delta in DELTAS {
            let dest = (center.0 + delta.0, center.1 + delta.1);
            assert!(board.is_features_match(&dest, &tile_city));
            board.set(dest, tile_city.clone());
        }

        assert!(board.is_features_match(&center, &tile_city));
        let mut bag = TileBag::default();
        for _ in 0..1_000_00 {
            if let Some(tile) = bag.pull() {
                if tile_city.matches_minis(&tile) {
                    assert!(board.is_features_match(&center, &tile));
                } else {
                    assert!(!board.is_features_match(&center, &tile));
                }
            } else {
                break;
            }
        }
    }
}
