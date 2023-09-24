use rand::{rngs::ThreadRng, Rng};

use crate::{
    arena::MessageResult,
    tile::{MiniTile, TileData, TileDataBuilder},
};

pub struct TileBag {
    data: Vec<TileData>,
    rng: ThreadRng,
    next_idx: NextTileType,
}

enum NextTileType {
    FirstTile(TileData),
    BagTile(usize),
    Empty,
}

impl TileBag {
    pub fn peek(&self) -> MessageResult<&TileData> {
        match &self.next_idx {
            NextTileType::FirstTile(tile) => Ok(tile),
            NextTileType::BagTile(idx) => Ok(&self.data[*idx]),
            NextTileType::Empty => Err("Empty bag"),
        }
    }

    pub fn pull(&mut self) -> Option<TileData> {
        let out = match &self.next_idx {
            NextTileType::FirstTile(tile) => Some(tile.clone()),
            NextTileType::BagTile(idx) => Some(self.data.swap_remove(*idx)),
            NextTileType::Empty => None,
        };
        self.pick_next_idx();
        out
    }

    fn pick_next_idx(&mut self) {
        if self.data.is_empty() {
            self.next_idx = NextTileType::Empty;
        } else {
            self.next_idx = NextTileType::BagTile(self.rng.gen_range(0..self.data.len()));
        }
    }

    pub fn count_remaining(&self) -> u32 {
        let offset = if let NextTileType::FirstTile(_) = self.next_idx {
            1
        } else {
            0
        };
        self.data.len() as u32 + offset
    }
}

impl Default for TileBag {
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

        TileBag {
            data: data.into_iter().map(|builder| builder.into()).collect(),
            rng: rand::thread_rng(),
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
        let mut bag = TileBag::default();
        assert_eq!(bag.count_remaining(), 72);
        bag.pull();
        assert_eq!(bag.count_remaining(), 71);
        bag.peek();
        assert_eq!(bag.count_remaining(), 71);
    }

    #[test]
    fn check_empties() {
        let mut bag = TileBag::default();
        let mut ct = 1_000_000;
        while bag.pull().is_some() {
            ct -= 1;
            assert!(ct > 0)
        }
    }
}
