use std::collections::HashMap;

use crate::{
    bot::Bot,
    referee::{Player, RefereeState},
};

pub struct Match {}

pub struct GameResult {}

pub type MessageResult<T> = Result<T, &'static str>;

impl Match {
    pub fn play(bots: Vec<Box<dyn Bot>>) -> MessageResult<GameResult> {
        let mut players: Vec<Player> = bots
            .iter()
            .map(|bot| bot.get_own_player().clone())
            .collect();
        players.sort();
        let mut state = RefereeState::from_players(players.clone());
        let mut player_map: HashMap<Player, Box<dyn Bot>> = HashMap::new();
        for bot in bots {
            player_map.insert(bot.get_own_player().clone(), bot);
        }

        while state.tilebag.peek().is_ok() {
            for turn in &players {
                let bot = player_map.get_mut(turn).unwrap();
                let move_request = bot.get_move(&state);
                state.process_move(move_request)?;
            }
        }

        Ok(GameResult {})
    }
}
