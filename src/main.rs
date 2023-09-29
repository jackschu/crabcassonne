use std::{
    path::PathBuf,
    rc::Rc,
    sync::{mpsc::channel, Mutex},
    thread::{self, JoinHandle},
};

use crabcassonne::{
    arena::{random_match, Match, Replay},
    bots::{bot::Bot, greedy_bot::GreedyBot, human_bot::HumanBot, random_bot::RandomBot},
    referee::Player,
    render::{InteractionMessage, MyApp, RenderMessage},
};

use clap::{Parser, Subcommand};
use rustc_hash::FxHashMap;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a game
    Play {
        /// Number of human players to start a game with
        #[arg(short, long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..3))]
        players: u8,
        /// Sets a destination file for replay
        #[arg(short, long, value_name = "REPLAY_FILE")]
        output: Option<PathBuf>,
    },
    /// Replay a replay file
    Replay {
        #[arg(short, long, value_name = "REPLAY_FILE")]
        input: PathBuf,
        #[arg(short, long, default_value_t = false)]
        headless: bool,
    },
    /// Evaluate bots
    Eval {
        #[command(subcommand)]
        demo: Demo,
    },
}

#[derive(Subcommand)]
enum Demo {
    /// [Benchmark] pits bots against eachother in multithreaded matches
    Threaded,
    /// [Benchmark] pits random-move bots against eachother in a single thread
    Random,
}

fn main() {
    //    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let cli = Cli::parse();
    match cli.command {
        Commands::Play { players, output } => demo_p(players, output),
        Commands::Replay { input, headless } => {
            let replay = Replay::from_path(input).unwrap();
            let result = replay.replay(!headless);
            result.print(FxHashMap::default());
        }
        Commands::Eval { demo } => match demo {
            Demo::Threaded => demo_threaded(),
            Demo::Random => random_match(1_000),
        },
    }
}

fn demo_p(player_ct: u8, record: Option<PathBuf>) {
    let (input_sender, input_receiver) = channel::<RenderMessage>();
    let (sender, receiver) = channel::<InteractionMessage>();

    let handle = thread::spawn(move || {
        let receiver_mutex = Rc::new(Mutex::new(receiver));
        let bot_w: Box<dyn Bot> = if player_ct > 0 {
            Box::new(HumanBot::new(
                Player::White,
                receiver_mutex.clone(),
                input_sender.clone(),
            ))
        } else {
            Box::new(GreedyBot::new(Player::White))
        };

        let bot_b: Box<dyn Bot> = if player_ct > 1 {
            Box::new(HumanBot::new(Player::Black, receiver_mutex, input_sender))
        } else {
            Box::new(RandomBot::new(Player::Black))
        };

        let mut names = FxHashMap::default();
        names.insert(Player::Black, bot_b.get_name().to_owned());
        names.insert(Player::White, bot_w.get_name().to_owned());

        Match::play(vec![bot_w, bot_b], record)
            .unwrap()
            .print(names);
    });

    if player_ct > 0 {
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
    handle.join().unwrap();
}

fn demo_threaded() {
    let desired_n = 10_000;
    let t = 8;
    let n_t = desired_n / t;
    let n = n_t * t;
    let mut threads: Vec<JoinHandle<(u32, u32, u32)>> = vec![];

    let get_white = || -> Box<dyn Bot> { Box::new(RandomBot::new(Player::White)) };
    let get_black = || -> Box<dyn Bot> { Box::new(GreedyBot::new(Player::Black)) };
    for _i in 0..t {
        let x = thread::spawn(move || {
            let mut white_win = 0;
            let mut black_win = 0;
            let mut draw = 0;
            for _i in 0..(n_t) {
                let bot_w = get_white();
                let bot_b = get_black();
                let result = Match::play(vec![bot_w, bot_b], None).unwrap();
                let winners = result.get_winners();
                if winners.len() > 1 {
                    draw += 1;
                } else if winners.into_iter().last().unwrap() == Player::White {
                    white_win += 1;
                } else {
                    black_win += 1;
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
        "White ({}) winrate {:.2}, Draw-rate {:.2}, Black ({}) winrate {:.2} (n = {n})",
        get_white().get_name(),
        white_win as f64 / n as f64,
        draw as f64 / n as f64,
        get_black().get_name(),
        black_win as f64 / n as f64,
    );
}
