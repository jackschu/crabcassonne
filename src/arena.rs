use std::{
    collections::{HashMap, HashSet},
    println,
};

use itertools::Itertools;

use crate::{
    board::BoardData,
    bot::Bot,
    referee::{Player, RefereeState},
};

pub struct Match {}

pub struct GameResult {
    pub player_scores: HashMap<Player, u32>,
}

impl GameResult {
    pub fn get_winners(&self) -> HashSet<Player> {
        let mut max_score: i32 = -1;
        let mut winners: HashSet<Player> = HashSet::new();
        for (player, score) in &self.player_scores {
            if max_score < *score as i32 {
                max_score = *score as i32;
                winners = HashSet::from([player.clone()]);
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
        let mut players: Vec<Player> = bots
            .iter()
            .map(|bot| bot.get_own_player().clone())
            .unique()
            .collect();
        players.sort();
        let mut state = RefereeState::from_players(players.clone());
        let mut player_map: HashMap<Player, Box<dyn Bot>> = HashMap::new();
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
