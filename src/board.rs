use std::{cmp::max, cmp::min};

use crate::{
    arena::MessageResult,
    referee::Player,
    tile::{MiniTile, Rotation, TileClickTarget, TileData, CARDINALS},
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rustc_hash::{FxHashMap, FxHashSet};

pub type Coordinate = (i8, i8);

#[derive(Clone, Default)]
pub struct ConcreteBoard {
    data: FxHashMap<Coordinate, TileData>,
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
static DELTAS_MAP: Lazy<FxHashMap<TileClickTarget, Coordinate>> = Lazy::new(|| {
    let mut out = FxHashMap::default();

    out.insert(TileClickTarget::Right, (0, 1));
    out.insert(TileClickTarget::Bottom, (1, 0));
    out.insert(TileClickTarget::Top, (-1, 0));
    out.insert(TileClickTarget::Left, (0, -1));
    out
});

// From unplaced tile to placed tile
static COUPLINGS: [(TileClickTarget, TileClickTarget); 4] = [
    (TileClickTarget::Right, TileClickTarget::Left),
    (TileClickTarget::Bottom, TileClickTarget::Top),
    (TileClickTarget::Top, TileClickTarget::Bottom),
    (TileClickTarget::Left, TileClickTarget::Right),
];

static COUPLINGS_MAP: Lazy<FxHashMap<TileClickTarget, TileClickTarget>> = Lazy::new(|| {
    let mut out = FxHashMap::default();
    for (k, v) in COUPLINGS.clone() {
        out.insert(k, v);
    }
    out
});

#[derive(Clone, Debug)]
pub struct ScoringData {
    pub scoring_players: FxHashSet<Player>,
    pub points: u8,
    pub completed: bool,
    pub removal_candidate: FxHashSet<(Coordinate, TileClickTarget)>,
}

pub struct FeatureResult<'a> {
    pub originators: FxHashSet<TileClickTarget>,
    pub originator_coord: Coordinate,
    pub completed: bool,
    pub feature: MiniTile,
    pub visited: FxHashSet<(Coordinate, TileClickTarget)>,
    pub board: Box<&'a dyn BoardData>,
}

#[derive(Clone)]
pub struct OverlaidBoard<'a> {
    pub board: Box<&'a dyn BoardData>,
    pub overlay: FxHashMap<Coordinate, &'a TileData>,
}

impl BoardData for OverlaidBoard<'_> {
    fn with_overlay<'a>(&'a self, coord: Coordinate, tile: &'a TileData) -> OverlaidBoard {
        let mut overlay = FxHashMap::default();
        overlay.insert(coord, tile);
        OverlaidBoard {
            board: Box::new(self),
            overlay,
        }
    }
    fn as_user(&self) -> BoardUser {
        BoardUser {
            board: Box::new(self),
        }
    }
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
    pub fn get_scoring_players(&self) -> FxHashSet<Player> {
        let mut counts: FxHashMap<Player, u8> = FxHashMap::default();
        let mut max_count = 0;

        for player in self.get_meeples() {
            let cur = *counts.get(&player).unwrap_or(&0);
            counts.insert(player, cur + 1);
            max_count = max(max_count, cur + 1);
        }
        let mut out: FxHashSet<Player> = FxHashSet::default();
        for (player, count) in counts {
            if count != max_count {
                continue;
            }
            out.insert(player);
        }
        out
    }
    pub fn as_scoring_data(&self, is_endgame: bool) -> ScoringData {
        let (scoring_players, points) = self.get_score(is_endgame);
        ScoringData {
            completed: self.completed,
            scoring_players,
            points,
            removal_candidate: self.get_removal_candidates(),
        }
    }

    pub fn get_meeples(&self) -> Vec<Player> {
        self.get_removal_candidates()
            .iter()
            .filter_map(|(coord, direction)| {
                let tile = self.board.at(coord)?;
                tile.get_meeple_at(direction)
            })
            .collect()
    }

    pub fn get_present_tiles(&self) -> impl Iterator<Item = &Coordinate> + '_ {
        self.visited
            .iter()
            .map(|(coord, _)| coord)
            .unique()
            .filter(|coord| self.board.at(coord).is_some())
    }

    fn get_removal_candidates(&self) -> FxHashSet<(Coordinate, TileClickTarget)> {
        let out: FxHashSet<(Coordinate, TileClickTarget)> = match self.feature {
            MiniTile::Road | MiniTile::City => self.visited.clone(),
            MiniTile::Monastery => {
                let mut out = FxHashSet::default();
                out.insert((self.originator_coord, TileClickTarget::Center));
                out
            }
            MiniTile::Grass | MiniTile::Junction => FxHashSet::default(),
        };
        out
    }

    pub fn get_score(&self, is_endgame: bool) -> (FxHashSet<Player>, u8) {
        if !is_endgame && !self.completed {
            return (FxHashSet::default(), 0);
        }
        let players = self.get_scoring_players();
        let score = match self.feature {
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
            MiniTile::Grass | MiniTile::Junction => 0,
        };
        (players, score)
    }
}

pub trait BoardData {
    fn at(&self, coord: &Coordinate) -> Option<&TileData>;
    fn tiles_present(&self) -> Box<dyn Iterator<Item = Coordinate> + '_>;
    fn as_user(&self) -> BoardUser;
    fn with_overlay<'a>(&'a self, coord: Coordinate, tile: &'a TileData) -> OverlaidBoard;
}

// seems like this weirdo pattern is needed (?) b/c I want to return a reference to
// board data (w/in feature result), so if this was just to be w/in board data
// that would be a reference to self which is not object safe?
pub struct BoardUser<'a> {
    pub board: Box<&'a dyn BoardData>,
}
impl BoardUser<'_> {
    pub fn tiles_placed(&self) -> u32 {
        self.board.tiles_present().count() as u32
    }

    pub fn get_connecting_feature_results(
        &self,
        initial_coord: &Coordinate,
        direction: TileClickTarget,
    ) -> Option<FeatureResult> {
        let initial_tile = self.board.at(initial_coord)?;
        let initial_feature = initial_tile.at(&direction);
        if initial_feature != &MiniTile::Road && initial_feature != &MiniTile::City {
            return None;
        }
        let mut complete = true;
        let mut queue = vec![(*initial_coord, direction.clone())];
        let mut visited = FxHashSet::default();
        visited.insert((*initial_coord, direction.clone()));
        while let Some((coord, direction)) = queue.pop() {
            if let Some(tile) = self.board.at(&coord) {
                let directions = tile.get_exits(&direction);
                for direction in &directions {
                    visited.insert((coord, direction.clone()).clone());
                }
                directions
                    .iter()
                    .map(|direction| {
                        (
                            ConcreteBoard::offset_coordinate(&coord, direction),
                            COUPLINGS_MAP.get(direction).unwrap().clone(),
                        )
                    })
                    .for_each(|elem| {
                        if visited.insert(elem.clone()) {
                            queue.push(elem);
                        }
                    });
            } else {
                complete = false;
            }
        }

        let keys: FxHashSet<TileClickTarget> = visited
            .iter()
            .filter(|(result_tile, _direction)| initial_coord == result_tile)
            .map(|(_, direction)| direction.clone())
            .collect();
        Some(FeatureResult {
            board: self.board.clone(),
            originators: keys,
            originator_coord: *initial_coord,
            completed: complete,
            feature: initial_feature.clone(),
            visited,
        })
    }

    pub fn get_standing_points(&self) -> FxHashMap<Player, u32> {
        let mut score_map: FxHashMap<Player, u32> = FxHashMap::default();
        let score_data = self.get_all_scoring_data();
        for data in score_data {
            for player in &data.scoring_players {
                if let Some(current) = score_map.get_mut(player) {
                    *current += data.points as u32;
                } else {
                    score_map.insert(player.clone(), data.points as u32);
                }
            }
        }
        score_map
    }

    pub fn get_all_scoring_data(&self) -> Vec<ScoringData> {
        let mut out: Vec<ScoringData> = vec![];
        let mut visited: FxHashSet<(Coordinate, TileClickTarget)> = FxHashSet::default();
        let meeple_tiles: Vec<(Coordinate, &TileData)> = self
            .board
            .tiles_present()
            .flat_map(|coord| Some((coord, self.board.at(&coord)?)))
            .collect();

        for (coord, tile) in meeple_tiles {
            let directions = tile.get_meeple_locations();

            for (direction, _player) in directions {
                if visited.contains(&(coord, direction.clone())) {
                    continue;
                }
                if let Some(result) = self.get_feature_result(&coord, &direction) {
                    let data = result.as_scoring_data(true);
                    for visit in &data.removal_candidate {
                        visited.insert(visit.clone());
                    }

                    out.push(data);
                }
            }
        }
        out
    }

    pub fn get_feature_score_data<'a>(
        &'a self,
        coord: &Coordinate,
        tile: &'a TileData,
    ) -> Vec<ScoringData> {
        let board = self.board.with_overlay(*coord, tile);

        let board_user = board.as_user();
        let mut data: Vec<FeatureResult> = vec![];
        // prevent double reporting single tile
        let mut seen: FxHashSet<TileClickTarget> = FxHashSet::default();
        for direction in &CARDINALS {
            if seen.get(direction).is_some() {
                continue;
            }
            let feature_result =
                board_user.get_connecting_feature_results(coord, direction.clone());
            seen.extend(
                feature_result
                    .as_ref()
                    .map(|x| &x.originators)
                    .unwrap_or(&FxHashSet::default())
                    .clone(),
            );
            if let Some(feature_result) = feature_result {
                data.push(feature_result);
            }
        }

        let mut monastery_checks: Vec<(&TileData, Coordinate)> = OCTAL_DELTAS
            .iter()
            .filter_map(|delta| {
                let coord = (delta.0 + coord.0, delta.1 + coord.1);
                Some((board.at(&coord)?, coord))
            })
            .collect();
        monastery_checks.push((tile, *coord));
        for (derived_tile, derived_coord) in monastery_checks {
            if derived_tile.center_matches(&MiniTile::Monastery) {
                let maybe_result = board_user
                    .get_monastery_feature_result(&derived_coord, &TileClickTarget::Center);
                if let Some(feature_result) = maybe_result {
                    data.push(feature_result);
                }
            }
        }
        data.iter().map(|x| x.as_scoring_data(false)).collect()
    }

    pub fn get_feature_result(
        &self,
        coord: &Coordinate,
        direction: &TileClickTarget,
    ) -> Option<FeatureResult> {
        let tile = self.board.at(coord)?;
        let feature = tile.at(direction);

        match feature {
            MiniTile::City | MiniTile::Road => {
                self.get_connecting_feature_results(coord, direction.clone())
            }
            MiniTile::Grass | MiniTile::Junction => None,
            MiniTile::Monastery => self.get_monastery_feature_result(coord, direction),
        }
    }

    pub fn get_points_from_score_data(
        &self,
        scores: &Vec<ScoringData>,
    ) -> FxHashMap<Option<Player>, u8> {
        let mut out: FxHashMap<Option<Player>, u8> = FxHashMap::default();
        for datum in scores {
            let players = &datum.scoring_players;
            let score = datum.points;
            if players.is_empty() {
                let cur = out.get(&None).unwrap_or(&0);
                out.insert(None, cur + score);
            }
            for player in players {
                if let Some(cur) = out.get_mut(&Some(player.clone())) {
                    *cur += score;
                } else {
                    out.insert(Some(player.clone()), score);
                }
            }
        }

        out
    }

    #[allow(dead_code)]
    fn get_completion_points(
        &self,
        coord: &Coordinate,
        tile: &TileData,
    ) -> FxHashMap<Option<Player>, u8> {
        let scores = self.get_feature_score_data(coord, tile);
        self.get_points_from_score_data(&scores)
    }

    fn get_monastery_feature_result(
        &self,
        initial_coord: &Coordinate,
        direction: &TileClickTarget,
    ) -> Option<FeatureResult> {
        let feature = self
            .board
            .at(initial_coord)
            .map(|tile| tile.at(direction))?;
        if feature != &MiniTile::Monastery {
            return None;
        }
        let mut completed = true;
        let mut out: FxHashSet<(Coordinate, TileClickTarget)> = FxHashSet::default();
        out.insert((*initial_coord, TileClickTarget::Center));

        for delta in OCTAL_DELTAS {
            let coord = (initial_coord.0 + delta.0, initial_coord.1 + delta.1);
            completed = completed && self.board.at(&coord).is_some();
            out.insert((coord, TileClickTarget::Center));
        }

        let mut originators = FxHashSet::default();
        originators.insert(TileClickTarget::Center);
        Some(FeatureResult {
            board: self.board.clone(),
            originators,
            originator_coord: *initial_coord,
            completed,
            feature: MiniTile::Monastery,
            visited: out,
        })
    }

    pub fn is_legal_meeple(
        &self,
        coord: &Coordinate,
        target: TileClickTarget,
    ) -> MessageResult<()> {
        let tile = self
            .board
            .at(coord)
            .ok_or("Illegal meeple: No tile present")?;
        let mini_feature = tile.at(&target);
        match mini_feature {
            MiniTile::City | MiniTile::Road => {
                if target == TileClickTarget::Center {
                    return Err("Cant place meeple on center for non monestary");
                }
                let feature_result = self.get_connecting_feature_results(coord, target)
                    .ok_or("Invariant violation in connecting feature result, likely called on nonconnecting feature")?;
                if !feature_result.visited.is_empty() && feature_result.get_meeples().is_empty() {
                    Ok(())
                } else {
                    Err("Illegal meeple: Feature is non empty")
                }
            }
            MiniTile::Grass | MiniTile::Junction => Err("Illegal meeple: Non scoring feature"),
            MiniTile::Monastery => {
                if tile.get_meeple_at(&target).is_some() {
                    Err("Illegal meeple: Feature is non empty")
                } else {
                    Ok(())
                }
            }
        }
    }
    pub fn does_legal_move_exist(&self, tile: &TileData) -> bool {
        if self.board.tiles_present().count() == 0 {
            return true;
        }
        let coords = self.get_legal_tiles();

        for coord in coords {
            for rotation in [
                Rotation::None,
                Rotation::Left,
                Rotation::Flip,
                Rotation::Right,
            ] {
                let mut tile = tile.clone();
                tile.rotation = rotation.clone();
                if self.is_features_match(&coord, &tile) {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_legal_moves(&self, tile: &TileData) -> Vec<(Coordinate, Rotation)> {
        let mut coords = self.get_legal_tiles();
        if self.board.tiles_present().count() == 0 {
            coords.insert((0, 0));
        }
        let mut out: Vec<(Coordinate, Rotation)> = vec![];
        for coord in coords {
            for rotation in [
                Rotation::None,
                Rotation::Left,
                Rotation::Flip,
                Rotation::Right,
            ] {
                let mut tile = tile.clone();
                tile.rotation = rotation.clone();
                if self.is_features_match(&coord, &tile) {
                    out.push((coord, rotation.clone()))
                }
            }
        }

        out
    }

    pub fn get_legal_tiles(&self) -> FxHashSet<Coordinate> {
        self.board
            .tiles_present()
            .flat_map(|tile| {
                DELTAS.iter().filter_map(move |delta| {
                    let coord = (delta.0 + tile.0, delta.1 + tile.1);
                    if self.contains_coord(&coord) {
                        None
                    } else {
                        Some(coord)
                    }
                })
            })
            .collect()
    }

    fn contains_coord(&self, coord: &Coordinate) -> bool {
        self.board.at(coord).is_some()
    }

    pub fn is_features_match(&self, dest: &Coordinate, incoming_tile: &TileData) -> bool {
        DELTAS
            .iter()
            .map(|delta| self.board.at(&(delta.0 + dest.0, delta.1 + dest.1)))
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
    fn as_user(&self) -> BoardUser {
        BoardUser {
            board: Box::new(self),
        }
    }
    fn with_overlay<'a>(&'a self, coord: Coordinate, tile: &'a TileData) -> OverlaidBoard {
        let mut overlay = FxHashMap::default();
        overlay.insert(coord, tile);
        OverlaidBoard {
            board: Box::new(self),
            overlay,
        }
    }
}

impl ConcreteBoard {
    pub fn boundaries(&self) -> ((i8, i8), (i8, i8)) {
        let last = self.data.iter().last();
        if let Some(last) = last {
            let (mut min_row, mut min_col) = last.0;
            let (mut max_row, mut max_col) = (min_row, min_col);
            for (row, col) in self.data.keys() {
                min_row = min(min_row, *row);
                min_col = min(min_col, *col);
                max_row = max(max_row, *row);
                max_col = max(max_col, *col);
            }

            ((min_row, max_row), (min_col, max_col))
        } else {
            ((1, -1), (1, -1))
        }
    }
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

    pub fn remove_meeples(
        &mut self,
        set: FxHashSet<(Coordinate, TileClickTarget)>,
    ) -> FxHashMap<Player, u8> {
        let mut return_meeples: FxHashMap<Player, u8> = FxHashMap::default();
        for (coord, target) in set {
            if let Some(tile) = self.data.get_mut(&coord) {
                if let Some(player) = tile.clear_meeple(&target) {
                    if let Some(output_count) = return_meeples.get_mut(&player) {
                        *output_count += 1;
                    } else {
                        return_meeples.insert(player, 1);
                    }
                }
            }
        }
        return_meeples
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
        assert_eq!(board.as_user().get_legal_tiles().len(), 4);
        board.set((29, 30), bag.pull().unwrap());
        assert_eq!(board.as_user().get_legal_tiles().len(), 6);
        board.set((29, 30), bag.pull().unwrap());
        assert_eq!(board.as_user().get_legal_tiles().len(), 6);

        board.set((29, 29), bag.pull().unwrap());
        assert_eq!(board.as_user().get_legal_tiles().len(), 7);

        board.set((30, 29), bag.pull().unwrap());
        assert_eq!(board.as_user().get_legal_tiles().len(), 8);
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
        assert!(!board.as_user().is_features_match(&(30, 31), &tile_right));

        tile_right.rotate_right();
        assert!(!board.as_user().is_features_match(&(30, 31), &tile_right));
        tile_right.rotate_right();

        assert!(board.as_user().is_features_match(&(30, 31), &tile_right));
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
            board
                .as_user()
                .get_completion_points(
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
                .get(&None)
                .unwrap_or(&0)
                .clone()
        );
    }

    #[test]
    fn complete_monastery() {
        let mut board = ConcreteBoard::default();

        let mut tile_monastery: TileData = TileDataBuilder {
            center: MiniTile::Monastery,
            ..Default::default()
        }
        .into();
        let success = tile_monastery.place_meeple(&TileClickTarget::Center, &Player::Black);
        assert!(success.is_ok());
        board.set((30, 30), tile_monastery);

        let mut tile: TileData = TileDataBuilder {
            ..Default::default()
        }
        .into();

        let success = tile.place_meeple(&TileClickTarget::Center, &Player::White);
        assert!(success.is_ok());

        board.set((30, 31), tile.clone());
        board.set((29, 31), tile.clone());
        board.set((31, 31), tile.clone());

        board.set((30, 29), tile.clone());
        board.set((31, 29), tile.clone());
        board.set((29, 29), tile.clone());

        board.set((29, 30), tile.clone());

        assert_eq!(
            board
                .as_user()
                .get_completion_points(&(31, 30), &tile)
                .get(&Some(Player::Black))
                .unwrap_or(&0)
                .clone(),
            9
        );

        let score_data = board.as_user().get_feature_score_data(&(31, 30), &tile);
        let mut removed = 0;
        for data in score_data {
            if !data.completed {
                continue;
            }
            let out = board.remove_meeples(data.removal_candidate);
            removed += out.get(&Player::Black).unwrap_or(&0);
            removed += out.get(&Player::White).unwrap_or(&0);
        }

        assert_eq!(removed, 1);
    }

    #[test]
    fn contested_road() {
        let mut board = ConcreteBoard::default();

        let mut tile: TileData = TileDataBuilder {
            top: MiniTile::Road,
            ..Default::default()
        }
        .into();

        let success = tile.place_meeple(&TileClickTarget::Top, &Player::Black);
        assert!(success.is_ok());
        board.set((0, 0), tile.clone());

        tile.rotate_right();
        board.set((-1, -1), tile.clone());

        let mut tile: TileData = TileDataBuilder {
            top: MiniTile::Road,
            ..Default::default()
        }
        .into();
        tile.rotate_left();
        let success = tile.place_meeple(&TileClickTarget::Left, &Player::White);
        assert!(success.is_ok());

        board.set((-1, 1), tile.clone());

        let mut completion_tile: TileData = TileDataBuilder {
            top: MiniTile::Road,
            bottom: MiniTile::Road,
            right: MiniTile::Road,
            center: MiniTile::Road,
            ..Default::default()
        }
        .into();
        completion_tile.rotate_right();

        let score_data = board
            .as_user()
            .get_feature_score_data(&(-1, 0), &completion_tile);
        assert_eq!(score_data.len(), 1);

        let datum = &score_data[0];
        let mut test_set = FxHashSet::default();
        test_set.insert(Player::Black);
        assert_eq!(datum.scoring_players, test_set);
        assert_eq!(datum.points, 4);
    }

    #[test]
    fn simple_score_no_rot() {
        let mut board = ConcreteBoard::default();

        let tile_city: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::Road,
            right: MiniTile::Road,
            center: MiniTile::Road,
            ..Default::default()
        }
        .into();
        board.set((1, 0), tile_city);
        let player = Player::White;

        let maybe_tile = board.at_mut(&(1, 0));
        if let Some(tile) = maybe_tile {
            let success = tile.place_meeple(&TileClickTarget::Top, &player);
            assert!(success.is_ok());
            let success = tile.place_meeple(&TileClickTarget::Left, &player);
            assert!(success.is_ok());
        }

        let tile_city: TileData = TileDataBuilder {
            bottom: MiniTile::City,
            ..Default::default()
        }
        .into();
        board.set((0, 0), tile_city);

        let tile = board.at(&(0, 0)).unwrap();
        let completion = board.as_user().get_completion_points(&(0, 0), tile);
        let points = *completion.get(&Some(player.clone())).unwrap_or(&0);
        assert_eq!(points, 4);

        let score_data = board.as_user().get_feature_score_data(&(0, 0), tile);
        let completion = board.as_user().get_points_from_score_data(&score_data);
        let points = *completion.get(&Some(player.clone())).unwrap_or(&0);
        assert_eq!(points, 4);

        let mut removed = 0;
        for data in score_data {
            let out = board.remove_meeples(data.removal_candidate);
            removed += out.get(&player).unwrap_or(&0);
            println!("{out:?}");
        }

        assert_eq!(removed, 1);
    }

    #[test]
    fn simple_score() {
        let mut board = ConcreteBoard::default();

        let tile_city: TileData = TileDataBuilder {
            top: MiniTile::City,
            ..Default::default()
        }
        .into();
        board.set((1, 0), tile_city);

        let mut tile_city: TileData = TileDataBuilder {
            top: MiniTile::Road,
            left: MiniTile::Road,
            center: MiniTile::Road,
            right: MiniTile::City,
            ..Default::default()
        }
        .into();
        tile_city.rotate_right();
        board.set((0, 0), tile_city.clone());
        let player = Player::White;
        let maybe_tile = board.at_mut(&(0, 0));
        if let Some(tile) = maybe_tile {
            let success = tile.place_meeple(&TileClickTarget::Bottom, &player);
            assert!(success.is_ok());
            let success = tile.place_meeple(&TileClickTarget::Right, &player);
            assert!(success.is_ok());
        }

        let tile = board.at(&(0, 0)).unwrap();
        let completion = board.as_user().get_completion_points(&(0, 0), tile);
        let points = *completion.get(&Some(player.clone())).unwrap_or(&0);
        assert_eq!(points, 4);

        let score_data = board.as_user().get_feature_score_data(&(0, 0), tile);
        let completion = board.as_user().get_points_from_score_data(&score_data);
        let points = *completion.get(&Some(player.clone())).unwrap_or(&0);
        assert_eq!(points, 4);

        let mut removed = 0;
        for data in score_data {
            if !data.completed {
                continue;
            }
            let out = board.remove_meeples(data.removal_candidate);
            removed += out.get(&player).unwrap_or(&0);
        }

        assert_eq!(removed, 1);
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
            assert!(board.as_user().is_features_match(&dest, &tile_city));
            board.set(dest, tile_city.clone());
        }

        assert!(board.as_user().is_features_match(&center, &tile_city));
        assert_eq!(
            board
                .as_user()
                .get_completion_points(&center, &tile_city)
                .get(&None)
                .unwrap_or(&0)
                .clone(),
            16
        );
        let mut bag = TileBag::default();
        for _ in 0..100_000 {
            if let Some(tile) = bag.pull() {
                if tile_city.matches_minis(&tile) {
                    assert!(board.as_user().is_features_match(&center, &tile));
                } else {
                    assert!(!board.as_user().is_features_match(&center, &tile));
                }
            } else {
                break;
            }
        }
    }
}
