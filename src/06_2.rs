use helpers::{run, State, TextManager, Timings};
use speedy2d::Graphics2D;

mod helpers;

struct MyState {}

impl State for MyState {
    fn on_draw(
        &mut self,
        _timings: &Timings,
        _text_manager: &mut TextManager,
        _graphics: &mut Graphics2D,
    ) {
    }
}

struct Run {
    time: u64,
    distance: u64,
}

fn main() {
    let mut score = 1;
    for run in &input() {
        let mut winning_move = 0;
        for x in 1..run.time {
            let distance = (run.time - x) * x;
            if distance >= run.distance {
                winning_move += 1;
            }
        }

        score *= winning_move;
    }

    dbg!(score);

    run(MyState {});
}

fn input() -> Vec<Run> {
    vec![Run {
        time: 35937366,
        distance: 212206012011044,
    }]
}
