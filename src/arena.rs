use std::println;

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    board::{BoardData, Coordinate},
    bot::{Bot, MoveRequest, ReplayBot},
    referee::{Player, RefereeState},
    tile::{Rotation, TileClickTarget, TileData},
    tilebag::{LegalTileBag, ReplayTileBag, TileBag},
};

pub struct Match {}

pub struct Replay {
    turn_order: Vec<Player>,
    moves: Vec<ConcreteMove>,
}

impl Replay {
    pub fn replay(&self) {
        let n = self.turn_order.len();
        let mut bots: Vec<ReplayBot> = vec![];
        for player in &self.turn_order {
            bots.push(ReplayBot::unitialized(player.clone()))
        }

        let mut i = 0;
        let mut bag_data: Vec<TileData> = vec![];
        for one_move in &self.moves {
            let bot = bots.get_mut(i % n).unwrap();
            bag_data.push(one_move.tile_data.clone());
            bot.add_move(one_move.into());
            i += 1;
        }
        let bots: Vec<Box<dyn Bot>> = bots
            .into_iter()
            .map(|x| -> Box<dyn Bot> { Box::new(x) })
            .collect();

        Match::play_custom(bots, Box::new(ReplayTileBag::new(bag_data)))
            .unwrap()
            .print();
    }
}

pub struct ConcreteMove {
    pub tile_data: TileData,
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub location: Option<TileClickTarget>,
}

impl Into<MoveRequest> for &ConcreteMove {
    fn into(self) -> MoveRequest {
        MoveRequest {
            coord: self.coord.clone(),
            rotation: self.rotation.clone(),
            meeple: self.location.clone(),
        }
    }
}
pub struct GameResult {
    pub player_scores: FxHashMap<Player, u32>,
}

impl GameResult {
    pub fn get_winners(&self) -> FxHashSet<Player> {
        let mut max_score: i32 = -1;
        let mut winners: FxHashSet<Player> = FxHashSet::default();
        for (player, score) in &self.player_scores {
            if max_score < *score as i32 {
                max_score = *score as i32;
                let mut new_winners = FxHashSet::default();
                new_winners.insert(player.clone());
                winners = new_winners;
            } else if max_score == *score as i32 {
                winners.insert(player.clone());
            }
        }
        winners
    }

    pub fn print(&self) {
        let winners = self.get_winners();
        println!("Winners {winners:?}");
        let scores: Vec<(Player, u32)> = self
            .player_scores
            .clone()
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
            .collect();
        for (player, score) in scores {
            println!("Player: {player} Score: {score}");
        }
    }
}

pub type MessageResult<T> = Result<T, &'static str>;

impl Match {
    pub fn play(bots: Vec<Box<dyn Bot>>) -> MessageResult<GameResult> {
        Self::play_custom(bots, Box::new(LegalTileBag::default()))
    }
    pub fn play_custom(
        bots: Vec<Box<dyn Bot>>,
        bag: Box<dyn TileBag>,
    ) -> MessageResult<GameResult> {
        let mut players: Vec<Player> = bots
            .iter()
            .map(|bot| bot.get_own_player().clone())
            .unique()
            .collect();
        players.sort();
        let mut state = RefereeState::from_players(players.clone(), bag);
        let mut player_map: FxHashMap<Player, Box<dyn Bot>> = FxHashMap::default();
        for bot in bots {
            player_map.insert(bot.get_own_player().clone(), bot);
        }

        while state.tilebag.peek().is_ok() {
            for turn in &players {
                state.tilebag.ensure_legal_draw(&state.board.as_user());
                let bot = player_map.get_mut(turn).unwrap();
                let move_request = bot.get_move(&state);
                state.process_move(move_request)?;
            }
        }

        let mut scores = state.board.as_user().get_standing_points();
        for player in players {
            let delta = state.player_scores.get(&player).unwrap_or(&0);
            if let Some(score) = scores.get_mut(&player) {
                *score += *delta
            } else {
                scores.insert(player, *delta);
            }
        }

        Ok(GameResult {
            player_scores: scores,
        })
    }
}
