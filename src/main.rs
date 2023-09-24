use std::{
    rc::Rc,
    sync::{mpsc::channel, Mutex},
    thread,
};

use crabcassonne::{
    arena::Match,
    bot::{Bot, HumanBot, RandomBot},
    referee::Player,
    render::{InteractionMessage, MyApp, RenderMessage},
};

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    demo_1p();
}

#[allow(dead_code, unused_must_use)]
fn demo_2p() {
    let (input_sender, input_receiver) = channel::<RenderMessage>();
    let (sender, receiver) = channel::<InteractionMessage>();

    thread::spawn(move || {
        let receiver_mutex = Rc::new(Mutex::new(receiver));
        let bot_w: Box<dyn Bot> = Box::new(HumanBot::new(
            Player::White,
            receiver_mutex.clone(),
            input_sender.clone(),
        ));
        let bot_b: Box<dyn Bot> =
            Box::new(HumanBot::new(Player::Black, receiver_mutex, input_sender));

        Match::play(vec![bot_w, bot_b]);
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
}

#[allow(dead_code, unused_must_use)]
fn demo_0p() {
    let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
    let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));
    Match::play(vec![bot_w, bot_b]);
}

#[allow(dead_code, unused_must_use)]
fn demo_1p() {
    let (input_sender, input_receiver) = channel::<RenderMessage>();
    let (sender, receiver) = channel::<InteractionMessage>();

    thread::spawn(move || {
        let bot_w: Box<dyn Bot> = Box::new(HumanBot::new(
            Player::White,
            Rc::new(Mutex::new(receiver)),
            input_sender,
        ));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));

        Match::play(vec![bot_w, bot_b]);
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
}
