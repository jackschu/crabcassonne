use rand::{rngs::ThreadRng, Rng};

use crate::referee::{MiniTile, TileData};

pub struct TileBag {
    data: Vec<TileData>,
    rng: ThreadRng,
    is_first: bool,
}

impl TileBag {
    pub fn pull(&mut self) -> TileData {
        if self.is_first {
            self.is_first = !self.is_first;
            return TileData {
                top: MiniTile::City,
                left: MiniTile::Road,
                bottom: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
        }
        let idx = self.rng.gen_range(0..self.data.len());
        self.data.swap_remove(idx)
    }
}

impl Default for TileBag {
    fn default() -> Self {
        let mut data: Vec<TileData> = Vec::new();

        // --------
        // 0 roads
        // --------
        data.append(&mut vec![
            TileData {
                center: MiniTile::Monastery,
                ..Default::default()
            };
            4
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                ..Default::default()
            };
            5
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                bottom: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                right: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileData {
                left: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            1
        ]);

        data.append(&mut vec![
            TileData {
                has_emblem: true,
                left: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileData {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                left: MiniTile::City,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileData {
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
            TileData {
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
            TileData {
                center: MiniTile::Monastery,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            2
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                right: MiniTile::City,
                center: MiniTile::City,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            1
        ]);

        data.append(&mut vec![
            TileData {
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
            TileData {
                left: MiniTile::Road,
                right: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            8
        ]);

        data.append(&mut vec![
            TileData {
                left: MiniTile::Road,
                center: MiniTile::Road,
                bottom: MiniTile::Road,
                ..Default::default()
            };
            9
        ]);

        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                left: MiniTile::Road,
                right: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);
        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                left: MiniTile::Road,
                bottom: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);
        data.append(&mut vec![
            TileData {
                top: MiniTile::City,
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            3
        ]);

        data.append(&mut vec![
            TileData {
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
            TileData {
                has_emblem: true,
                top: MiniTile::City,
                right: MiniTile::City,
                secondary_center: Some(MiniTile::City),
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Road,
                ..Default::default()
            };
            2
        ]);

        // --------
        // 3 roads
        // -------
        data.append(&mut vec![
            TileData {
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Junction,
                ..Default::default()
            };
            4
        ]);
        data.append(&mut vec![
            TileData {
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
            TileData {
                top: MiniTile::Road,
                right: MiniTile::Road,
                bottom: MiniTile::Road,
                left: MiniTile::Road,
                center: MiniTile::Junction,
                ..Default::default()
            };
            1
        ]);

        assert_eq!(data.len(), 71);

        TileBag {
            data,
            rng: rand::thread_rng(),
            is_first: true,
        }
    }
}
