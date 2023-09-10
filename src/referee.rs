use std::sync::mpsc::Receiver;

use crate::render::Message;

const BOARD_SIZE: usize = 72 * 72;

pub struct Board {
    data: [Tile; BOARD_SIZE],
}

#[derive(Copy, Clone)]
enum Tile {
    PlacedTile(PlacedTile),
    EmptyTile,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            data: [Tile::EmptyTile; BOARD_SIZE],
        }
    }
}

#[derive(Copy, Clone)]
pub struct PlacedTile {
    pub has_emblem: bool,
    data: [MiniTile; 5],
}

#[derive(Copy, Clone)]
enum MiniTile {
    Grass,
    Road,
    City,
    Monastery,
}

pub fn referee_main(receiver: Receiver<Message>) {
    let board = Board::default();
    loop {
        match receiver.recv().unwrap() {
            Message::PrintMessage(message) => {
                println!("recv {}", message)
            }
        }
    }
}
