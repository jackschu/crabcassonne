use enum_dispatch::enum_dispatch;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    arena::MessageResult,
    board::OverlaidBoard,
    tile::{MiniTile, TileData, TileDataBuilder},
};

#[derive(Clone)]
pub enum NextTileType {
    BagTile(usize),
    Empty,
}

#[enum_dispatch]
pub enum TileBagEnum {
    LegalTileBag,
    ReplayTileBag,
}

#[enum_dispatch(TileBagEnum)]
pub trait TileBag: Sync {
    fn rig_idx_last(&mut self);

    fn rig(&mut self, rig: Vec<TileData>) {
        for elem in &rig {
            let data = self.get_data_mut();
            let mut found = false;
            for i in 0..data.len() {
                if data[i].matches_minis(elem) {
                    data.swap_remove(i);
                    found = true;
                    break;
                }
            }
            if !found {
                println!("Failed to find in tilebag rig");
            }
        }

        if let Some(tile) = rig.last() {
            self.get_data_mut().push(tile.clone());
            self.rig_idx_last();
        }
    }

    fn get_data_mut(&mut self) -> &mut Vec<TileData>;

    fn get_data(&self) -> &Vec<TileData>;
    fn peek(&self) -> MessageResult<&TileData> {
        match &self.get_next_idx() {
            NextTileType::BagTile(idx) => Ok(&self.get_data()[*idx]),
            NextTileType::Empty => Err("Empty bag"),
        }
    }

    fn pull(&mut self) -> Option<TileData> {
        let out = match &self.get_next_idx() {
            NextTileType::BagTile(idx) => {
                let idx = *idx;
                Some(self.get_data_mut().swap_remove(idx))
            }
            NextTileType::Empty => None,
        };
        self.pick_next_idx();
        out
    }

    fn as_new_tile_bag(&self) -> LegalTileBag {
        let data = self.get_data().clone();
        LegalTileBag::from_data(data, self.get_next_idx().clone())
    }

    // discards tiles until game is legal
    // @returns True: is legal draw possible or False: we are out of tiles
    fn ensure_legal_draw(&mut self, board_user: &OverlaidBoard) -> bool {
        while let Ok(tile) = self.peek() {
            let legal = board_user.does_legal_move_exist(tile);
            if legal {
                return true;
            }
            self.pull();
        }
        false
    }
    fn pick_next_idx(&mut self);
    fn get_next_idx(&self) -> &NextTileType;

    fn count_remaining(&self) -> u32 {
        self.get_data().len() as u32
    }
}

pub struct LegalTileBag {
    data: Vec<TileData>,
    rng: StdRng,
    next_idx: NextTileType,
}

impl TileBag for LegalTileBag {
    fn get_data_mut(&mut self) -> &mut Vec<TileData> {
        &mut self.data
    }
    fn get_data(&self) -> &Vec<TileData> {
        &self.data
    }
    fn get_next_idx(&self) -> &NextTileType {
        &self.next_idx
    }
    fn rig_idx_last(&mut self) {
        self.next_idx = NextTileType::BagTile(self.data.len() - 1);
    }
    fn pick_next_idx(&mut self) {
        if self.data.is_empty() {
            self.next_idx = NextTileType::Empty;
        } else {
            self.next_idx = NextTileType::BagTile(self.rng.gen_range(0..self.data.len()));
        }
    }
}

pub struct ReplayTileBag {
    pub data: Vec<TileData>,
    next_idx: NextTileType,
}
impl ReplayTileBag {
    pub fn new(mut data: Vec<TileData>) -> Self {
        data.reverse();
        let idx = data.len() - 1;
        Self {
            data,
            next_idx: NextTileType::BagTile(idx),
        }
    }
}

impl TileBag for ReplayTileBag {
    fn get_data_mut(&mut self) -> &mut Vec<TileData> {
        &mut self.data
    }
    fn get_data(&self) -> &Vec<TileData> {
        &self.data
    }
    fn get_next_idx(&self) -> &NextTileType {
        &self.next_idx
    }
    fn rig_idx_last(&mut self) {
        self.next_idx = NextTileType::BagTile(self.data.len() - 1);
    }
    fn pick_next_idx(&mut self) {
        if self.data.is_empty() {
            self.next_idx = NextTileType::Empty;
        } else {
            self.next_idx = NextTileType::BagTile(self.data.len() - 1);
        }
    }
}

impl LegalTileBag {
    fn from_data(data: Vec<TileData>, next: NextTileType) -> Self {
        Self {
            data,
            rng: StdRng::seed_from_u64(rand::random()),
            next_idx: next,
        }
    }
}

impl Default for LegalTileBag {
    fn default() -> Self {
        let mut data: Vec<TileDataBuilder> = Vec::new();

        // --------
        // 0 roads
        // --------
        data.append(&mut vec![
            TileDataBuilder {
                center: MiniTile::Monastery,
                ..Default::default()
            };
            4
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                ..Default::default()
            };
            5
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                bottom: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                left: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            1
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                left: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                left: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                left: MiniTile::City,
                ..Default::default()
            };
            1
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                left: MiniTile::City,
                bottom: MiniTile::City,
                ..Default::default()
            };
            1
        ]);

        // --------
        // 1 roads
        // --------
        data.append(&mut vec![
            TileDataBuilder {
                center: MiniTile::Monastery,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            1
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            2
        ]);

        // --------
        // 2 roads
        // --------

        data.append(&mut vec![
            TileDataBuilder {
                left: MiniTile::Road,
                right: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            8
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                left: MiniTile::Road,
                center: MiniTile::Road,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            9
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                left: MiniTile::Road,
                right: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);
        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                left: MiniTile::Road,
                bottom: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);
        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::City,
                secondary_center: Some(MiniTile::City),
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);
        data.append(&mut vec![
            TileDataBuilder {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                secondary_center: Some(MiniTile::City),
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Road,
            };
            2
        ]);

        // --------
        // 3 roads
        // -------
        data.append(&mut vec![
            TileDataBuilder {
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Junction,
                ..Default::default()
            };
            4
        ]);
        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::City,
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Junction,
                ..Default::default()
            };
            3
        ]);

        // --------
        // 4 roads
        // -------
        data.append(&mut vec![
            TileDataBuilder {
                top: MiniTile::Road,
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Junction,
                ..Default::default()
            };
            1
        ]);

        data.push(
            TileDataBuilder {
                top: MiniTile::City,
                left: MiniTile::Road,
                right: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            }
            .into(),
        );
        // for debug endgame
        // let out: Vec<TileDataBuilder> = data.into_iter().take(5).collect();
        let data: Vec<TileData> = data.into_iter().map(|builder| builder.into()).collect();

        let idx = data.len() - 1;
        LegalTileBag {
            data,
            rng: StdRng::seed_from_u64(rand::random()),
            next_idx: NextTileType::BagTile(idx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unused_must_use)]
    fn check_count() {
        let mut bag = LegalTileBag::default();
        assert_eq!(bag.count_remaining(), 72);
        bag.pull();
        assert_eq!(bag.count_remaining(), 71);
        bag.peek();
        assert_eq!(bag.count_remaining(), 71);
    }

    #[test]
    fn check_empties() {
        let mut bag = LegalTileBag::default();
        let mut ct = 1_000_000;
        while bag.pull().is_some() {
            ct -= 1;
            assert!(ct > 0)
        }
    }
}
