use std::{
    rc::Rc,
    sync::{mpsc::channel, Mutex},
    thread::{self, JoinHandle},
};

use crabcassonne::{
    arena::Match,
    bot::{Bot, HumanBot, RandomBot},
    referee::Player,
    render::{InteractionMessage, MyApp, RenderMessage},
};

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    demo_0p();
}

#[allow(dead_code)]
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

        Match::play(vec![bot_w, bot_b]).unwrap().print();
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

#[allow(dead_code)]
fn demo_0p() {
    let desired_n = 100000;
    let t = 8;
    let n_t = desired_n / t;
    let n = n_t * t;
    let mut threads: Vec<JoinHandle<(u32, u32, u32)>> = vec![];

    for _i in 0..t {
        let x = thread::spawn(move || {
            let mut white_win = 0;
            let mut black_win = 0;
            let mut draw = 0;
            for _i in 0..(n_t) {
                let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
                let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));
                let result = Match::play(vec![bot_w, bot_b]).unwrap();
                let winners = result.get_winners();
                if winners.len() > 1 {
                    draw += 1;
                } else {
                    if winners.into_iter().last().unwrap() == Player::White {
                        white_win += 1;
                    } else {
                        black_win += 1;
                    }
                }
            }
            (white_win, draw, black_win)
        });
        threads.push(x)
    }

    let mut white_win = 0;
    let mut black_win = 0;
    let mut draw = 0;

    for thread in threads {
        let (w_d, d_d, b_d) = thread.join().unwrap();
        white_win += w_d;
        draw += d_d;
        black_win += b_d;
    }

    println!(
        "White winrate {:.2}, Draw-rate {:.2}, Black winrate {:.2} (n = {n})",
        white_win as f64 / n as f64,
        draw as f64 / n as f64,
        black_win as f64 / n as f64,
    );
}

#[allow(dead_code)]
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

        Match::play(vec![bot_w, bot_b]).unwrap().print();
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
