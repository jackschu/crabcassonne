use std::panic;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    arena::MessageResult,
    board::BoardUser,
    tile::{MiniTile, TileData, TileDataBuilder},
};

#[derive(Clone)]
pub enum NextTileType {
    FirstTile(TileData),
    BagTile(usize),
    Empty,
}

pub trait TileBag: Sync {
    fn get_data_mut(&mut self) -> &mut Vec<TileData>;

    fn get_data(&self) -> &Vec<TileData>;
    fn peek(&self) -> MessageResult<&TileData> {
        match &self.get_next_idx() {
            NextTileType::FirstTile(tile) => Ok(tile),
            NextTileType::BagTile(idx) => Ok(&self.get_data()[*idx]),
            NextTileType::Empty => Err("Empty bag"),
        }
    }

    fn pull(&mut self) -> Option<TileData> {
        let out = match &self.get_next_idx() {
            NextTileType::FirstTile(tile) => Some(tile.clone()),
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

    fn ensure_legal_draw(&mut self, board_user: &BoardUser) {
        for _i in 0..1000 {
            if let Ok(tile) = self.peek() {
                let legal = board_user.does_legal_move_exist(tile);
                if legal {
                    return;
                }
                self.pick_next_idx();
            } else {
                return;
            }
        }
        panic!("Couldnt find legal draw");
    }
    fn pick_next_idx(&mut self);
    fn get_next_idx(&self) -> &NextTileType;

    fn count_remaining(&self) -> u32 {
        let offset = if let NextTileType::FirstTile(_) = self.get_next_idx() {
            1
        } else {
            0
        };
        self.get_data().len() as u32 + offset
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
        let front = data.pop().unwrap();
        Self {
            data,
            next_idx: NextTileType::FirstTile(front),
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

        // for debug endgame
        // let out: Vec<TileDataBuilder> = data.into_iter().take(5).collect();

        LegalTileBag {
            data: data.into_iter().map(|builder| builder.into()).collect(),
            rng: StdRng::seed_from_u64(rand::random()),
            next_idx: NextTileType::FirstTile(
                TileDataBuilder {
                    top: MiniTile::City,
                    left: MiniTile::Road,
                    right: MiniTile::Road,
                    center: MiniTile::Road,
                    ..Default::default()
                }
                .into(),
            ),
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
