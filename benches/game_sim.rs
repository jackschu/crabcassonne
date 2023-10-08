use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Note RAYON_NUM_THREADS=1 may be useful here

use crabcassonne::{
    arena::{random_match, Match},
    bots::{bot::Bot, greedy_bot::GreedyBot, shallow_bot::ShallowBot},
    referee::Player,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn greedy_match(n: u64) {
    for _i in 0..n {
        let bot_w: Box<dyn Bot> = Box::new(GreedyBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(GreedyBot::new(Player::Black));
        let result = Match::play(vec![bot_w, bot_b], None).unwrap();
        let _winners = result.get_winners();
    }
}

fn shallow_match(n: u64) {
    for _i in 0..n {
        let bot_w: Box<dyn Bot> = Box::new(ShallowBot::new(Player::White, 10));
        let bot_b: Box<dyn Bot> = Box::new(ShallowBot::new(Player::Black, 10));
        let result = Match::play(vec![bot_w, bot_b], None).unwrap();
        let _winners = result.get_winners();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("games 100", |b| b.iter(|| random_match(black_box(100))));
    c.bench_function("greedy_bot 100", |b| {
        b.iter(|| greedy_match(black_box(100)))
    });
}

fn slow_bots(c: &mut Criterion) {
    c.bench_function("shallow_bot 1", |b| b.iter(|| shallow_match(black_box(1))));
}

criterion_group!(name = benches; config =  Criterion::default(); targets = criterion_benchmark);
criterion_group!(name = slow_benches; config =  Criterion::default().sample_size(10); targets = slow_bots);
criterion_main!(benches, slow_benches);
