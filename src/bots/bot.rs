use crate::{
    board::Coordinate,
    referee::{Player, RefereeState},
    tile::{Rotation, TileClickTarget},
};

pub trait Bot {
    fn get_own_player(&self) -> &Player;
    fn get_move(&mut self, state: &RefereeState) -> MoveRequest;
}

#[derive(Clone)]
pub struct MoveRequest {
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub meeple: Option<TileClickTarget>,
}
