use core::panic;

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
}

impl Default for CoordMap {
    fn default() -> Self {
        let mut v = Vec::new();
        v.resize_with(72 * 72, Default::default);
        Self {
            data: v,
            coords_present: Vec::with_capacity(72),
        }
    }
}
impl CoordMap {
    fn coord_to_key(&self, k: &Coordinate) -> usize {
        let first = (k.0 + 36) as i32 * 72;
        let out = first as usize + (k.1 + 36) as usize;
        if out > 72 * 72 {
            panic!("Unimplemented resize logic");
        }
        out
    }
    fn key_to_coord(&self, k: usize) -> Coordinate {
        let rem = k % 72;
        let second: i8 = rem as i8 - 36;
        let first: i8 = ((k - rem) / 72) as i8 - 36;
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
        let key = self.coord_to_key(k);
        self.data.get(key).unwrap_or(&None).as_ref()
    }
    pub fn get_mut(&mut self, k: &Coordinate) -> Option<&mut TileData> {
        let key = self.coord_to_key(k);
        if let Some(out) = self.data.get_mut(key) {
            return out.as_mut();
        }
        None
    }
    pub fn insert(&mut self, k: Coordinate, v: TileData) {
        let key = self.coord_to_key(&k);
        if self.data[key].is_none() {
            self.coords_present.push(k)
        }
        self.data[key] = Some(v);
    }
}
