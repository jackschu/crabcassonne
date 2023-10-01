use std::{
    path::PathBuf,
    rc::Rc,
    sync::{mpsc::channel, Mutex},
    thread::{self, JoinHandle},
};

use crabcassonne::{
    arena::{random_match, Match, Replay},
    bots::{bot::Bot, greedy_bot::GreedyBot, human_bot::HumanBot, shallow_bot::ShallowBot},
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
        #[arg(long, default_value_t = false)]
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
    Threaded {
        #[arg(short, long, default_value_t = 10_000)]
        num_games: u32,
    },
    /// [Benchmark] pits random-move bots against eachother in a single thread
    Random {
        #[arg(short, long, default_value_t = 10_000)]
        num_games: u32,
    },
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
            Demo::Threaded { num_games } => demo_threaded(num_games),
            Demo::Random { num_games } => random_match(num_games.into()),
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
            Box::new(ShallowBot::new(Player::Black))
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

#[derive(Default)]
struct AggStats {
    pub white_win: u32,
    pub draw: u32,
    pub black_win: u32,
    pub white_advantage: i32,
}

impl AggStats {
    pub fn get_n(&self) -> u32 {
        self.white_win + self.draw + self.black_win
    }
    pub fn add(&mut self, other: Self) {
        self.white_advantage += other.white_advantage;
        self.white_win += other.white_win;
        self.draw += other.draw;
        self.black_win += other.black_win;
    }
}

fn demo_threaded(desired_n: u32) {
    let t = 8;
    let n_t = desired_n / t;
    let n = n_t * t;
    if n != desired_n {
        println!("WARN, requested games {desired_n}, not divisible by threads {t} n instead {n}");
    }
    let mut threads: Vec<JoinHandle<AggStats>> = vec![];

    let get_white = || -> Box<dyn Bot> { Box::new(ShallowBot::new(Player::White)) };
    let get_black = || -> Box<dyn Bot> { Box::new(GreedyBot::new(Player::Black)) };
    for _i in 0..t {
        let x = thread::spawn(move || {
            let mut stats = AggStats::default();
            for _i in 0..(n_t) {
                let bot_w = get_white();
                let bot_b = get_black();
                let result = Match::play(vec![bot_w, bot_b], None).unwrap();
                stats.white_advantage += *result.player_scores.get(&Player::White).unwrap() as i32;
                stats.white_advantage -= *result.player_scores.get(&Player::Black).unwrap() as i32;
                let winners = result.get_winners();
                if winners.len() > 1 {
                    stats.draw += 1;
                } else if winners.into_iter().last().unwrap() == Player::White {
                    stats.white_win += 1;
                } else {
                    stats.black_win += 1;
                }
            }
            stats
        });
        threads.push(x)
    }

    let mut stats = AggStats::default();

    for thread in threads {
        stats.add(thread.join().unwrap());
    }

    let n = stats.get_n();
    println!(
        "White ({}) winrate {:.2}, Draw-rate {:.2}, Black ({}) winrate {:.2}\nwhite avg score advantage = {:.1} (n = {n})",
        get_white().get_name(),
        stats.white_win as f64 / n as f64,
        stats.draw as f64 / n as f64,
        get_black().get_name(),
        stats.black_win as f64 / n as f64,
        stats.white_advantage as f64 / n as f64,
    );
}
