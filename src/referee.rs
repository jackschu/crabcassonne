const BOARD_SIZE: usize = 72 * 72;

pub struct Board {
    data: [Tile; BOARD_SIZE],
}

pub struct Tile {
    pub has_emblem: bool,
    data: [MiniTile; 5],
}

enum MiniTile {
    Grass,
    Road,
    City,
    Monastery,
}
