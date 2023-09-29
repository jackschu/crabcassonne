use crabcassonne::{
    arena::{random_match, Match},
    bots::{bot::Bot, greedy_bot::GreedyBot},
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("games 100", |b| b.iter(|| random_match(black_box(100))));
    c.bench_function("greedy_bot 100", |b| {
        b.iter(|| greedy_match(black_box(100)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
