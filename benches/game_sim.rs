use crabcassonne::{
    arena::Match,
    bot::{Bot, RandomBot},
    referee::Player,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn random_match(n: u64) {
    for _i in 0..n {
        let bot_w: Box<dyn Bot> = Box::new(RandomBot::new(Player::White));
        let bot_b: Box<dyn Bot> = Box::new(RandomBot::new(Player::Black));
        let result = Match::play(vec![bot_w, bot_b]).unwrap();
        let _winners = result.get_winners();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("games 100", |b| b.iter(|| random_match(black_box(100))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
