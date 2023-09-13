use std::sync::mpsc::{Receiver, Sender};

use crate::{
    board::Board,
    render::{InteractionMessage, RenderMessage},
    tilebag::TileBag,
};

pub fn referee_main(receiver: Receiver<InteractionMessage>, sender: Sender<RenderMessage>) {
    let mut board = Board::default();
    let mut tilebag = TileBag::default();
    if let Some(tile) = tilebag.peek() {
        sender
            .send(RenderMessage::PreviewTile(tile.clone()))
            .unwrap();
    }
    loop {
        sender.send(RenderMessage::NewBoard(board.clone())).unwrap();
        match receiver.recv().unwrap() {
            InteractionMessage::Print(message) => {
                println!("recv {}", message);
            }
            InteractionMessage::Click(message) => {
                if board.tiles_placed() != 0 {
                    let legal_tiles = board.get_legal_tiles();
                    if !legal_tiles.contains(&(message.coord)) {
                        continue;
                    }
                    let maybe_next = tilebag.peek();
                    if let Some(next) = maybe_next {
                        let mut next = next.clone();
                        next.rotation = message.rotation.clone();
                        if !board.is_features_match(&(message.coord), &next) {
                            continue;
                        }
                    }
                }
                if let Some(mut tile) = tilebag.pull() {
                    tile.rotation = message.rotation;
                    board.get_completion_points(&tile, &message.coord);
                    board.set(message.coord, tile);
                } else {
                    println!("out of tiles");
                }
                if let Some(tile) = tilebag.peek() {
                    sender
                        .send(RenderMessage::PreviewTile(tile.clone()))
                        .unwrap();
                }
            }
        }
    }
}
