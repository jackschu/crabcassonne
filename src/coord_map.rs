use rustc_hash::FxHashMap;

use crate::{board::Coordinate, tile::TileData};

#[derive(Clone, Default)]
pub struct ReferenceCoordMap {
    data: FxHashMap<Coordinate, TileData>,
}

impl ReferenceCoordMap {
    pub fn iter(&self) -> impl Iterator<Item = (&Coordinate, &TileData)> {
        self.data.iter()
    }
    pub fn get(&self, k: &Coordinate) -> Option<&TileData> {
        self.data.get(k)
    }
    pub fn get_mut(&mut self, k: &Coordinate) -> Option<&mut TileData> {
        self.data.get_mut(k)
    }
    pub fn insert(&mut self, k: Coordinate, v: TileData) {
        self.data.insert(k, v);
    }
}

#[derive(Clone)]
pub struct CoordMap {
    data: Vec<Option<TileData>>,
    coords_present: Vec<Coordinate>,
    inner_size: usize,
}

// must be divisible by 2
const INITIAL_BOARD: usize = 12;
const INC: usize = 4;
impl Default for CoordMap {
    fn default() -> Self {
        let mut v = Vec::new();
        v.resize_with(INITIAL_BOARD * INITIAL_BOARD, Default::default);
        Self {
            data: v,
            coords_present: Vec::with_capacity(INITIAL_BOARD),
            inner_size: INITIAL_BOARD,
        }
    }
}
impl CoordMap {
    pub fn get_inner_size(&self) -> usize {
        self.inner_size
    }
    fn is_oob(&self, k: i32) -> bool {
        let lim = (self.inner_size / 2) as i32;
        k >= lim || k < -lim
    }
    fn coord_to_key(&self, k: &Coordinate) -> Option<usize> {
        if self.is_oob(k.0 as i32) || self.is_oob(k.1 as i32) {
            return None;
        }
        let first = (k.0 + (self.inner_size / 2) as i8) as i32 * (self.inner_size as i32);
        let second = k.1 as i32 + (self.inner_size / 2) as i32;
        let out = (first + second) as usize;
        if out >= (self.inner_size * self.inner_size) {
            None
        } else {
            Some(out)
        }
    }
    fn key_to_coord(&self, k: usize) -> Coordinate {
        let rem = k % self.inner_size;
        let second: i8 = rem as i8 - (self.inner_size / 2) as i8;
        let first: i8 =
            ((k - rem) / self.inner_size) as i8 - (self.inner_size / 2) as i8;
        (first, second)
    }
    pub fn tiles_present(&self) -> impl Iterator<Item = Coordinate> + '_ {
        self.coords_present.iter().cloned()
    }
    pub fn iter(&self) -> impl Iterator<Item = (Coordinate, &TileData)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(key, data)| data.as_ref().map(|data| (self.key_to_coord(key), data)))
    }
    pub fn get(&self, k: &Coordinate) -> Option<&TileData> {
        let key = self.coord_to_key(k)?;
        self.data.get(key)?.as_ref()
    }
    pub fn get_mut(&mut self, k: &Coordinate) -> Option<&mut TileData> {
        let key = self.coord_to_key(k)?;
        if let Some(out) = self.data.get_mut(key) {
            return out.as_mut();
        }
        None
    }

    fn debug_print(&self) {
        let lim: i32 = (self.inner_size / 2) as i32;
        for i in -lim..lim {
            for j in -lim..lim {
                print!(
                    "{}",
                    if self.get(&(j as i8, i as i8)).is_none() {
                        0
                    } else {
                        1
                    }
                )
            }
            println!();
        }
    }

    fn resize(&mut self) {
        let old_size = self.inner_size;
        self.inner_size += INC;
        let mut pre_post = Vec::new();
        pre_post.resize_with(
            (INC / 2) * old_size + 2 * (INC / 2) * (INC / 2) + INC / 2,
            Default::default,
        );

        let mut between = Vec::new();
        between.resize_with(INC, Default::default);

        self.data.reserve(self.inner_size * self.inner_size);
        self.data.splice(0..0, pre_post.clone());

        let mut idx = pre_post.len();
        for _ in 0..(old_size - 1) {
            idx += old_size;
            self.data.splice(idx..idx, between.clone());
            idx += between.len();
        }

        self.data.extend_from_slice(&pre_post);
    }
    pub fn insert(&mut self, k: Coordinate, v: TileData) {
        let mut key = self.coord_to_key(&k);
        while key.is_none() {
            self.resize();
            key = self.coord_to_key(&k);
        }
        let key = key.unwrap();
        if self.data[key].is_none() {
            self.coords_present.push(k)
        }
        self.data[key] = Some(v);
        self.debug_print();
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::tile::{MiniTile, TileDataBuilder};

    use super::*;
    #[test]
    fn resize_test() {
        let mut map = CoordMap::default();
        assert_eq!(map.get_inner_size(), INITIAL_BOARD);

        let tile1: TileData = TileDataBuilder {
            ..Default::default()
        }
        .into();
        let coord1 = (0, 0);
        map.insert(coord1, tile1.clone());

        let tile2: TileData = TileDataBuilder {
            right: MiniTile::City,
            ..Default::default()
        }
        .into();

        let coord2 = (INITIAL_BOARD as i8 / 2, INITIAL_BOARD as i8 / 2);
        map.insert(coord2, tile2.clone());

        assert!(map.get(&coord1).unwrap().matches_minis(&tile1));
        assert!(map.get(&coord2).unwrap().matches_minis(&tile2));
    }
}
