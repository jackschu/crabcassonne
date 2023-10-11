use crate::tilebag::TileBag;
use std::cmp::Ordering;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::referee::{Player, RefereeState};

use super::bot::{Bot, MoveRequest};

pub struct GreedyBot {
    pub own_player: Player,
    rng: ThreadRng,
}

impl GreedyBot {
    pub fn new(player: Player) -> Self {
        GreedyBot {
            own_player: player,
            rng: rand::thread_rng(),
        }
    }
}

impl Bot for GreedyBot {
    fn get_name(&self) -> String {
        "greedy bot".to_owned()
    }

    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let board_user = state.board.as_overlay();

        let tile = state.tilebag.peek().unwrap();
        let can_place = state
            .player_meeples
            .get(self.get_own_player())
            .map(|ct| ct > &0)
            .unwrap_or(false);
        let moves: Vec<MoveRequest> = board_user.get_legal_moves(tile, can_place);
        let mut candidate: Option<(MoveRequest, i32)> = None;
        for move_request in moves {
            let mut tile = tile.clone();
            if let Some(location) = &move_request.meeple {
                tile.place_meeple(location, self.get_own_player()).unwrap();
            }
            tile.rotation = move_request.rotation.clone();
            let points = board_user.get_completion_points(&move_request.coord, &tile);
            let mut total: i32 = 0;
            for (player, points) in points {
                if let Some(player) = player {
                    if &player == self.get_own_player() {
                        total += points as i32;
                    } else {
                        total -= points as i32;
                    }
                }
            }
            if let Some((_request, score)) = candidate.clone() {
                match score.cmp(&total) {
                    Ordering::Less => {
                        candidate = Some((move_request.clone(), total));
                    }
                    Ordering::Equal => {
                        if self.rng.gen_bool(0.5) {
                            candidate = Some((move_request.clone(), total));
                        }
                    }
                    Ordering::Greater => {}
                }
            } else {
                candidate = Some((move_request.clone(), total));
            }
        }
        candidate.unwrap().0
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::{
        bots::random_bot::RandomBot,
        tile::{MiniTile, TileData, TileDataBuilder},
        tilebag::ReplayTileBag,
    };
    #[test]
    fn is_greedy() {
        let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));

        let bots = vec![bot_w, bot_b];
        let mut players: Vec<Player> = bots
            .iter()
            .map(|bot| bot.get_own_player().clone())
            .unique()
            .collect();
        players.sort();
        let first: TileData = TileDataBuilder {
            top: MiniTile::City,
            left: MiniTile::Road,
            center: MiniTile::Road,
            right: MiniTile::Road,
            ..Default::default()
        }
        .into();
        let second: TileData = TileDataBuilder {
            right: MiniTile::Road,
            ..Default::default()
        }
        .into();
        let third: TileData = TileDataBuilder {
            left: MiniTile::Road,
            center: MiniTile::Junction,
            ..Default::default()
        }
        .into();
        let bag = ReplayTileBag::new(vec![first, second, third]);

        let mut state = RefereeState::from_players(players.clone(), bag.into());
        state
            .process_move(MoveRequest {
                coord: (0, 0),
                ..Default::default()
            })
            .unwrap();
        state
            .process_move(MoveRequest {
                coord: (0, -1),
                ..Default::default()
            })
            .unwrap();
        let mut bot_b: Box<dyn Bot> = Box::new(GreedyBot::new(Player::Black));
        for _i in 0..10 {
            let move_request = bot_b.get_move(&state);
            let expected = MoveRequest {
                coord: (0, 1),
                rotation: crate::tile::Rotation::None,
                meeple: Some(crate::tile::TileClickTarget::Left),
            };
            assert_eq!(move_request, expected);
        }
    }
}
