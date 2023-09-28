use crate::{
    board::Coordinate,
    referee::{Player, RefereeState},
    tile::{Rotation, TileClickTarget},
};

pub trait Bot {
    fn get_own_player(&self) -> &Player;
    fn get_name(&self) -> &str;
    fn get_move(&mut self, state: &RefereeState) -> MoveRequest;
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct MoveRequest {
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub meeple: Option<TileClickTarget>,
}
