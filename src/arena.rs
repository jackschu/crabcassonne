use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    println,
};

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    board::{BoardData, Coordinate},
    bot::{Bot, MoveRequest, ReplayBot},
    referee::{Player, RefereeState},
    tile::{Rotation, TileClickTarget, TileData},
    tilebag::{LegalTileBag, ReplayTileBag, TileBag},
};

use serde::{Deserialize, Serialize};

pub struct Match {}

#[derive(Deserialize, Serialize, Default)]
pub struct Replay {
    pub turn_order: Vec<Player>,
    pub moves: Vec<ConcreteMove>,
}

impl Replay {
    pub fn from_path(input: PathBuf) -> MessageResult<Self> {
        let file = File::open(input).or(Err("failed to open file"))?;

        let mut buf_reader = std::io::BufReader::new(file);

        // Read the JSON content from the file into a string
        let mut json_string = String::new();
        buf_reader
            .read_to_string(&mut json_string)
            .or(Err("failed to read file"))?;

        // Deserialize the JSON string into a MyStruct object
        serde_json::from_str(&json_string).or(Err("failed to deserialize file"))
    }
    pub fn replay(&self) -> GameResult {
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

        Match::play_custom(bots, Box::new(ReplayTileBag::new(bag_data)), None).unwrap()
    }
}

#[derive(Deserialize, Serialize)]
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

#[derive(Eq, PartialEq, Debug)]
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
    pub fn play(bots: Vec<Box<dyn Bot>>, record: Option<PathBuf>) -> MessageResult<GameResult> {
        Self::play_custom(bots, Box::new(LegalTileBag::default()), record)
    }
    pub fn play_custom(
        bots: Vec<Box<dyn Bot>>,
        bag: Box<dyn TileBag>,
        record: Option<PathBuf>,
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

        let mut replay_data = Replay::default();
        replay_data.turn_order = players.clone();
        while state.tilebag.peek().is_ok() {
            for turn in &players {
                state.tilebag.ensure_legal_draw(&state.board.as_user());
                let bot = player_map.get_mut(turn).unwrap();
                let move_request = bot.get_move(&state);
                if record.is_some() {
                    if let Ok(tile) = state.tilebag.peek() {
                        replay_data.moves.push(ConcreteMove {
                            tile_data: tile.clone(),
                            coord: move_request.coord.clone(),
                            rotation: move_request.rotation.clone(),
                            location: move_request.meeple.clone(),
                        });
                    }
                }
                state.process_move(move_request)?;
            }
        }
        if let Some(path) = record {
            let file = File::create(path).or(Err("Failed to create replay file"))?;
            let mut file_writer = std::io::BufWriter::new(file);
            let json_string =
                serde_json::to_string(&replay_data).or(Err("Failed to serialize replay"))?;

            file_writer
                .write(json_string.as_bytes())
                .or(Err("Failed to write to file"))?;
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
#[cfg(test)]
mod tests {

    use std::assert_eq;

    use crate::bot::RandomBot;

    use super::*;

    #[test]
    fn replay_works() {
        let path = PathBuf::from("test_path.replay");

        let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));

        let result = Match::play(vec![bot_w, bot_b], Some(path.clone())).unwrap();

        let replay = Replay::from_path(path).unwrap();
        let replay_result = replay.replay();
        assert_eq!(result, replay_result);
    }
}
