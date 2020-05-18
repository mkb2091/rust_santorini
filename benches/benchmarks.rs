use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_santorini::{Game, Status, TowerStates};

fn criterion_benchmark(c: &mut Criterion) {
    let game = Game {
        board: [[TowerStates::Empty; 5]; 5],
        player_locations: [((1, 1), (3, 3)), ((17, 17), (17, 17)), ((17, 17), (17, 17))],
        player_statuses: [Status::Playing, Status::Dead, Status::Dead],
    };
    c.bench_function("list_possible_actions_on_empty", |b| {
        b.iter(|| game.list_possible_actions(0))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);