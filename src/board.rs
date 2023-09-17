use std::collections::{HashMap, HashSet};

use crate::tile::{MiniTile, TileClickTarget, TileData, CARDINALS};
use itertools::Itertools;
use once_cell::sync::Lazy;

pub const BOARD_DIM: usize = 72;

pub type Coordinate = (i8, i8);

#[derive(Clone, Default)]
pub struct ConcreteBoard {
    data: HashMap<Coordinate, TileData>,
}

pub static DELTAS: [Coordinate; 4] = [(0, 1), (1, 0), (-1, 0), (0, -1)];
pub static OCTAL_DELTAS: [Coordinate; 8] = [
    (0, 1),
    (1, 0),
    (-1, 0),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];
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

pub struct FeatureResult<'a> {
    pub originators: HashSet<TileClickTarget>,
    pub completed: bool,
    pub feature: MiniTile,
    pub visited: HashSet<(Coordinate, TileClickTarget)>,
    pub board: Box<&'a dyn BoardData>,
}

#[derive(Clone)]
pub struct OverlaidBoard<'a> {
    pub board: Box<&'a dyn BoardData>,
    pub overlay: HashMap<Coordinate, &'a TileData>,
}

impl BoardData for OverlaidBoard<'_> {
    fn at(&self, coord: &Coordinate) -> Option<&TileData> {
        if let Some(tile) = self.overlay.get(coord) {
            Some(tile)
        } else {
            self.board.at(coord)
        }
    }
    fn tiles_present(&self) -> Box<dyn Iterator<Item = Coordinate> + '_> {
        Box::new(
            self.board
                .tiles_present()
                .chain(self.overlay.iter().map(|x| *x.0))
                .unique(),
        )
    }
}

impl FeatureResult<'_> {
    pub fn get_present_tiles(&self) -> impl Iterator<Item = &Coordinate> + '_ {
        self.visited
            .iter()
            .map(|(coord, _)| coord)
            .unique()
            .filter(|coord| self.board.at(coord).is_some())
    }
    pub fn get_score(&self, is_endgame: bool) -> u8 {
        if !is_endgame && !self.completed {
            return 0;
        }
        match self.feature {
            MiniTile::Road | MiniTile::City => {
                let is_city = self.feature == MiniTile::City;
                let multiplier: u8 = if !is_endgame && is_city { 2 } else { 1 };
                let mut unit_count = self.get_present_tiles().count() as u8;
                if is_city {
                    unit_count += self
                        .get_present_tiles()
                        .filter_map(|coord| self.board.at(coord))
                        .filter(|tile| tile.has_emblem)
                        .count() as u8;
                }
                unit_count * multiplier
            }
            MiniTile::Monastery => self.get_present_tiles().count() as u8,
            _ => 0,
        }
    }
}

pub trait BoardData {
    fn at(&self, coord: &Coordinate) -> Option<&TileData>;
    fn tiles_placed(&self) -> u32 {
        self.tiles_present().count() as u32
    }
    fn tiles_present(&self) -> Box<dyn Iterator<Item = Coordinate> + '_>;

    fn get_connecting_feature_results(
        &self,
        initial_coord: &Coordinate,
        direction: TileClickTarget,
    ) -> Option<FeatureResult>
    where
        Self: Sized,
    {
        let initial_tile = self.at(initial_coord)?;
        let initial_feature = initial_tile.at(&direction);
        let mut complete = true;
        let mut queue = vec![(*initial_coord, direction.clone())];
        let mut visited = HashSet::from([(*initial_coord, direction.clone())]);

        while let Some((coord, direction)) = queue.pop() {
            if let Some(tile) = self.at(&coord) {
                let directions = tile.get_exits(&direction);
                let next: Vec<(Coordinate, TileClickTarget)> = directions
                    .iter()
                    .map(|direction| {
                        (
                            ConcreteBoard::offset_coordinate(&coord, direction),
                            COUPLINGS_MAP.get(direction).unwrap().clone(),
                        )
                    })
                    .filter(|elem| visited.get(elem).is_none())
                    .collect();
                for elem in next {
                    visited.insert(elem.clone());
                    queue.push(elem);
                }
            } else {
                complete = false;
            }
        }

        let keys: HashSet<TileClickTarget> = visited
            .iter()
            .filter(|(result_tile, _direction)| initial_coord == result_tile)
            .map(|(_, direction)| direction.clone())
            .collect();
        Some(FeatureResult {
            board: Box::new(self),
            originators: keys,
            completed: complete,
            feature: initial_feature.clone(),
            visited,
        })
    }

    fn get_completion_points(&self, coord: &Coordinate, tile: &TileData) -> u8
    where
        Self: Sized,
    {
        let overlay: HashMap<Coordinate, &TileData> = HashMap::from([(*coord, tile)]);
        let board = OverlaidBoard {
            board: Box::new(self),
            overlay,
        };

        let mut data: Vec<FeatureResult> = vec![];
        // prevent double reporting single tile
        let mut seen: HashSet<TileClickTarget> = HashSet::from([]);
        for direction in &CARDINALS {
            if seen.get(direction).is_some() {
                continue;
            }
            let feature_result = board.get_connecting_feature_results(coord, direction.clone());
            for elem in feature_result
                .as_ref()
                .map(|x| &x.originators)
                .unwrap_or(&HashSet::from([]))
            {
                seen.insert(elem.clone());
            }
            if let Some(feature_result) = feature_result {
                data.push(feature_result);
            }
        }
        let mut out: u8 = 0;

        let mut monestary_checks: Vec<(&TileData, Coordinate)> = OCTAL_DELTAS
            .iter()
            .filter_map(|delta| {
                let coord = (delta.0 + coord.0, delta.1 + coord.1);
                Some((board.at(&coord)?, coord))
            })
            .collect();
        monestary_checks.push((tile, *coord));
        for (derived_tile, derived_coord) in monestary_checks {
            if derived_tile.center_matches(&MiniTile::Monastery) {
                let maybe_result =
                    board.get_monestary_feature_result(&derived_coord, &TileClickTarget::Center);
                if let Some(feature_result) = maybe_result {
                    data.push(feature_result);
                }
            }
        }

        for datum in data {
            let score = datum.get_score(false);
            println!("score {} feature {:?}", score, datum.feature);
            out += score;
        }

        out
    }

    fn get_monestary_feature_result(
        &self,
        initial_coord: &Coordinate,
        direction: &TileClickTarget,
    ) -> Option<FeatureResult>
    where
        Self: Sized,
    {
        let feature = self.at(initial_coord).map(|tile| tile.at(direction))?;
        if feature != &MiniTile::Monastery {
            return None;
        }
        let mut completed = true;
        let mut out: HashSet<(Coordinate, TileClickTarget)> = HashSet::from([]);
        out.insert((*initial_coord, TileClickTarget::Center));

        for delta in OCTAL_DELTAS {
            let coord = (initial_coord.0 + delta.0, initial_coord.1 + delta.1);
            completed = completed && self.at(&coord).is_some();
            out.insert((coord, TileClickTarget::Center));
        }
        Some(FeatureResult {
            board: Box::new(self),
            originators: HashSet::from([TileClickTarget::Center]),
            completed,
            feature: MiniTile::Monastery,
            visited: out,
        })
    }

    fn get_legal_tiles(&self) -> HashSet<Coordinate> {
        self.tiles_present()
            .flat_map(|tile| -> HashSet<Coordinate> {
                DELTAS
                    .iter()
                    .filter_map(|delta| {
                        let coord = (delta.0 + tile.0, delta.1 + tile.1);
                        if self.contains_coord(&coord) {
                            None
                        } else {
                            Some(coord)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn contains_coord(&self, coord: &Coordinate) -> bool {
        self.at(coord).is_some()
    }

    fn is_features_match(&self, dest: &Coordinate, incoming_tile: &TileData) -> bool {
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

impl BoardData for ConcreteBoard {
    fn at(&self, coord: &Coordinate) -> Option<&TileData> {
        self.data.get(coord)
    }
    fn tiles_present(&self) -> Box<dyn Iterator<Item = Coordinate> + '_> {
        Box::new(self.data.iter().map(|x| *x.0))
    }
}

impl ConcreteBoard {
    pub fn at_mut(&mut self, coord: &Coordinate) -> Option<&mut TileData> {
        self.data.get_mut(coord)
    }

    pub fn set(&mut self, coord: Coordinate, tile: TileData) {
        self.data.insert(coord, tile);
    }

    fn offset_coordinate(coord: &Coordinate, direction: &TileClickTarget) -> Coordinate {
        let offset = DELTAS_MAP.get(direction).unwrap();
        (coord.0 + offset.0, coord.1 + offset.1)
    }
    fn with_overlay<'a>(&'a self, coord: Coordinate, tile: &'a TileData) -> OverlaidBoard {
        OverlaidBoard {
            board: Box::new(self),
            overlay: HashMap::from([(coord, tile)]),
        }
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
        let mut board = ConcreteBoard::default();
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
        let mut board = ConcreteBoard::default();
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

    #[test]
    fn feature_completion_test() {
        let mut board = ConcreteBoard::default();

        board.set(
            (30, 30),
            TileDataBuilder {
                right: MiniTile::City,
                ..Default::default()
            }
            .into(),
        );
        board.set(
            (30, 31),
            TileDataBuilder {
                right: MiniTile::City,
                left: MiniTile::City,
                bottom: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            }
            .into(),
        );
        board.set(
            (31, 31),
            TileDataBuilder {
                top: MiniTile::City,
                ..Default::default()
            }
            .into(),
        );
        board.set(
            (29, 32),
            TileDataBuilder {
                bottom: MiniTile::City,
                ..Default::default()
            }
            .into(),
        );
        assert_eq!(
            12,
            board.get_completion_points(
                &(30, 32),
                &TileDataBuilder {
                    has_emblem: true,
                    top: MiniTile::City,
                    left: MiniTile::City,
                    center: MiniTile::City,
                    ..Default::default()
                }
                .into()
            )
        );
    }

    #[test]
    fn complete_monestary() {
        let mut board = ConcreteBoard::default();

        let tile_monestary: TileData = TileDataBuilder {
            center: MiniTile::Monastery,
            ..Default::default()
        }
        .into();
        board.set((30, 30), tile_monestary);

        let tile: TileData = TileDataBuilder {
            ..Default::default()
        }
        .into();

        board.set((30, 31), tile.clone());
        board.set((29, 31), tile.clone());
        board.set((31, 31), tile.clone());

        board.set((30, 29), tile.clone());
        board.set((31, 29), tile.clone());
        board.set((29, 29), tile.clone());

        board.set((29, 30), tile.clone());

        assert_eq!(board.get_completion_points(&(31, 30), &tile), 9);
    }

    #[test]
    fn features_match_surround_city() {
        let mut board = ConcreteBoard::default();

        let tile_city: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::City,
            right: MiniTile::City,
            bottom: MiniTile::City,
            ..Default::default()
        }
        .into();

        let center = (30_i8, 30_i8);
        for delta in DELTAS {
            let dest = (center.0 + delta.0, center.1 + delta.1);
            assert!(board.is_features_match(&dest, &tile_city));
            board.set(dest, tile_city.clone());
        }

        assert!(board.is_features_match(&center, &tile_city));
        assert_eq!(board.get_completion_points(&center, &tile_city), 16);
        let mut bag = TileBag::default();
        for _ in 0..100_000 {
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
