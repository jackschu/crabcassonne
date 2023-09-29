use std::cmp::Ordering;
use std::{cmp::max, cmp::min};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    println,
    sync::mpsc::channel,
    thread,
};

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::bots::random_bot::RandomBot;
use crate::{
    board::{BoardData, Coordinate},
    bots::{bot::Bot, bot::MoveRequest, replay_bot::ReplayBot},
    referee::{Player, RefereeState},
    render::{InteractionMessage, MyApp, RenderMessage, RenderState},
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

        let mut json_string = String::new();
        buf_reader
            .read_to_string(&mut json_string)
            .or(Err("failed to read file"))?;

        serde_json::from_str(&json_string).or(Err("failed to deserialize file"))
    }

    pub fn replay(&self, should_render: bool) -> GameResult {
        let n = self.turn_order.len();
        let mut bots: Vec<ReplayBot> = vec![];
        for player in &self.turn_order {
            bots.push(ReplayBot::unitialized(player.clone()))
        }

        let mut bag_data: Vec<TileData> = vec![];
        for (i, one_move) in self.moves.iter().enumerate() {
            let bot = bots.get_mut(i % n).unwrap();
            bag_data.push(one_move.tile_data.clone());
            bot.add_move(one_move.into());
        }
        let bots: Vec<Box<dyn Bot>> = bots
            .into_iter()
            .map(|x| -> Box<dyn Bot> { Box::new(x) })
            .collect();

        if !should_render {
            return Match::play_custom(bots, Box::new(ReplayTileBag::new(bag_data)), None, None)
                .unwrap();
        }
        let mut frames: Vec<RenderState> = vec![];
        let out = Match::play_custom(
            bots,
            Box::new(ReplayTileBag::new(bag_data)),
            None,
            Some(&mut frames),
        )
        .unwrap();
        self.replay_ui(frames);
        out
    }

    fn replay_ui(&self, frames: Vec<RenderState>) {
        let (input_sender, input_receiver) = channel::<RenderMessage>();
        let (sender, receiver) = channel::<InteractionMessage>();

        let mut frame_idx: isize = 0;
        let handle = thread::spawn(move || loop {
            input_sender
                .send(RenderMessage::RefereeSync(
                    frames[frame_idx as usize].clone(),
                ))
                .unwrap();
            match receiver.recv().unwrap() {
                InteractionMessage::NextFrame => {
                    frame_idx = min(frame_idx + 1, frames.len() as isize - 1);
                }
                InteractionMessage::PreviousFrame => {
                    frame_idx = max(frame_idx - 1, 0);
                }
                InteractionMessage::LastFrame => {
                    frame_idx = frames.len() as isize - 1;
                }
                InteractionMessage::FirstFrame => {
                    frame_idx = 0;
                }
                _ => {}
            }
        });
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1600.0, 900.0)),
            ..Default::default()
        };

        eframe::run_native(
            "Crabcassonne",
            options,
            Box::new(|_cc| Box::new(MyApp::create(sender, input_receiver))),
        )
        .unwrap();
        handle.join().unwrap();
    }
}

#[derive(Deserialize, Serialize)]
pub struct ConcreteMove {
    pub tile_data: TileData,
    pub coord: Coordinate,
    pub rotation: Rotation,
    pub location: Option<TileClickTarget>,
}

impl From<&ConcreteMove> for MoveRequest {
    fn from(val: &ConcreteMove) -> Self {
        MoveRequest {
            coord: val.coord,
            rotation: val.rotation.clone(),
            meeple: val.location.clone(),
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
            match max_score.cmp(&(*score as i32)) {
                Ordering::Less => {
                    max_score = *score as i32;
                    let mut new_winners = FxHashSet::default();
                    new_winners.insert(player.clone());
                    winners = new_winners;
                }
                Ordering::Equal => {
                    winners.insert(player.clone());
                }
                Ordering::Greater => {}
            }
        }
        winners
    }

    pub fn print(&self, names: FxHashMap<Player, String>) {
        let winners = self.get_winners();
        println!("Winners {winners:?}");
        let scores: Vec<(Player, u32)> = self
            .player_scores
            .clone()
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
            .collect();
        for (player, score) in scores {
            let default_name = &"unknown".to_owned();
            let name = names.get(&player).unwrap_or(default_name);
            println!("Player: {player} ({name}) Score: {score}");
        }
    }
}

pub type MessageResult<T> = Result<T, &'static str>;

impl Match {
    pub fn play(bots: Vec<Box<dyn Bot>>, record: Option<PathBuf>) -> MessageResult<GameResult> {
        Self::play_custom(bots, Box::<LegalTileBag>::default(), record, None)
    }
    pub fn play_custom(
        bots: Vec<Box<dyn Bot>>,
        bag: Box<dyn TileBag>,
        record: Option<PathBuf>,
        replay_frames: Option<&mut Vec<RenderState>>,
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

        let mut replay_data = Replay {
            turn_order: players.clone(),
            ..Default::default()
        };
        while state.tilebag.peek().is_ok() {
            for turn in &players {
                state.tilebag.ensure_legal_draw(&state.board.as_user());
                let bot = player_map.get_mut(turn).unwrap();
                let move_request = bot.get_move(&state);
                if record.is_some() {
                    if let Ok(tile) = state.tilebag.peek() {
                        replay_data.moves.push(ConcreteMove {
                            tile_data: tile.clone(),
                            coord: move_request.coord,
                            rotation: move_request.rotation.clone(),
                            location: move_request.meeple.clone(),
                        });
                    }
                }
                state.process_move(move_request)?;
                if let Some(&mut ref mut frames) = replay_frames {
                    frames.push(state.clone_into());
                }
            }
        }
        if let Some(path) = record {
            let file = File::create(path).or(Err("Failed to create replay file"))?;
            let mut file_writer = std::io::BufWriter::new(file);
            let json_string =
                serde_json::to_string(&replay_data).or(Err("Failed to serialize replay"))?;

            file_writer
                .write_all(json_string.as_bytes())
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

pub fn random_match(n: u64) {
    for _i in 0..n {
        let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));
        let result = Match::play(vec![bot_w, bot_b], None).unwrap();
        let _winners = result.get_winners();
    }
}

#[cfg(test)]
mod tests {

    use std::assert_eq;

    use crate::bots::random_bot::RandomBot;

    use super::*;

    #[test]
    fn replay_works() {
        let path = PathBuf::from("test_path.replay");

        let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));

        let result = Match::play(vec![bot_w, bot_b], Some(path.clone())).unwrap();

        let replay = Replay::from_path(path).unwrap();
        let replay_result = replay.replay(false);
        assert_eq!(result, replay_result);
    }
}
