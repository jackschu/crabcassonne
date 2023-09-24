use std::collections::HashMap;

use crate::{
    bot::Bot,
    referee::{Player, RefereeState},
};

struct Arena {}

struct GameResult {}

impl Arena {
    pub fn play(bots: HashMap<Player, Box<dyn Bot>>) -> GameResult {
        let mut state = RefereeState::from_bots(bots);

        GameResult {}
    }
}
