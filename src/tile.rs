use std::hash::Hash;

use egui::Color32;

#[derive(Clone, Default)]
pub struct TileDataBuilder {
    pub has_emblem: bool,
    pub top: MiniTile,
    pub left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    pub right: MiniTile,
    pub bottom: MiniTile,
}

#[derive(Clone, Debug)]
pub struct TileData {
    pub has_emblem: bool,
    top: MiniTile,
    left: MiniTile,
    pub center: MiniTile,
    pub secondary_center: Option<MiniTile>,
    right: MiniTile,
    bottom: MiniTile,
    pub rotation: Rotation,
}

#[derive(Clone, Debug)]
pub enum Rotation {
    None,
    Left,
    Right,
    Flip,
}

impl Rotation {
    pub fn next_right(&self) -> Rotation {
        match &self {
            Rotation::None => Rotation::Right,
            Rotation::Right => Rotation::Flip,
            Rotation::Flip => Rotation::Left,
            Rotation::Left => Rotation::None,
        }
    }
    pub fn next_left(&self) -> Rotation {
        match &self {
            Rotation::None => Rotation::Left,
            Rotation::Left => Rotation::Flip,
            Rotation::Flip => Rotation::Right,
            Rotation::Right => Rotation::None,
        }
    }
}

impl TileData {
    /**
     * @return true iff rotation respected cardinals match
     */
    pub fn matches_minis(&self, other: &TileData) -> bool {
        self.top() == other.top()
            && self.bottom() == other.bottom()
            && self.left() == other.left()
            && self.right() == other.right()
    }
    pub fn top(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.top,
            Rotation::Left => &self.right,
            Rotation::Right => &self.left,
            Rotation::Flip => &self.bottom,
        }
    }
    pub fn bottom(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.bottom,
            Rotation::Left => &self.left,
            Rotation::Right => &self.right,
            Rotation::Flip => &self.top,
        }
    }

    pub fn right(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.right,
            Rotation::Left => &self.bottom,
            Rotation::Right => &self.top,
            Rotation::Flip => &self.left,
        }
    }
    pub fn left(&self) -> &MiniTile {
        match self.rotation {
            Rotation::None => &self.left,
            Rotation::Left => &self.top,
            Rotation::Right => &self.bottom,
            Rotation::Flip => &self.right,
        }
    }
    pub fn rotate_right(&mut self) {
        self.rotation = self.rotation.next_right();
    }
    pub fn rotate_left(&mut self) {
        self.rotation = self.rotation.next_left();
    }
}

impl From<TileDataBuilder> for TileData {
    fn from(builder: TileDataBuilder) -> TileData {
        TileData {
            has_emblem: builder.has_emblem,
            top: builder.top,
            left: builder.left,
            center: builder.center,
            secondary_center: builder.secondary_center,
            right: builder.right,
            bottom: builder.bottom,
            rotation: Rotation::None,
        }
    }
}

impl TileData {
    pub fn at(&self, target: &TileClickTarget) -> &MiniTile {
        match target {
            TileClickTarget::Top => self.top(),
            TileClickTarget::Left => self.left(),
            TileClickTarget::Center => &self.center,
            TileClickTarget::Right => self.right(),
            TileClickTarget::Bottom => self.bottom(),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum TileClickTarget {
    Top,
    Left,
    Center,
    Right,
    Bottom,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MiniTile {
    #[default]
    Grass,
    Road,
    City,
    Monastery,
    Junction,
}

impl MiniTile {
    pub fn get_color(&self) -> Color32 {
        match self {
            Self::Grass => Color32::from_rgb(0, 188, 84),
            Self::Road => Color32::WHITE,
            Self::City => Color32::from_rgb(205, 137, 48),
            Self::Monastery => Color32::RED,
            Self::Junction => Color32::YELLOW,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn tile_rotation() {
        let mut tile: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::Road,
            right: MiniTile::Junction,
            bottom: MiniTile::Monastery,
            ..Default::default()
        }
        .into();
        let original = tile.clone();

        tile.rotate_left();
        tile.rotate_right();

        for _ in 0..4 {
            tile.rotate_right();
        }

        for _ in 0..4 {
            tile.rotate_left();
        }

        assert_eq!(original.top(), tile.top());
        assert_eq!(original.bottom(), tile.bottom());
        assert_eq!(original.left(), tile.left());
        assert_eq!(original.right(), tile.right());

        tile.rotate_right();

        assert_eq!(original.left(), tile.top());
        assert_eq!(original.right(), tile.bottom());
        assert_eq!(original.top(), tile.right());
        assert_eq!(original.bottom(), tile.left());

        tile.rotate_left();
        tile.rotate_left();

        assert_eq!(original.right(), tile.top());
        assert_eq!(original.left(), tile.bottom());
        assert_eq!(original.top(), tile.left());
        assert_eq!(original.bottom(), tile.right());
    }
}
