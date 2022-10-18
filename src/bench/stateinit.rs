use mandelox::bench::{Benchmark as B, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::state::{MbArrayState, MbVecState};

fn b_mbstate_init(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();

    let f = move || {
        MbArrayState::initialize(width, height, &grid);
    };
    B::iter(&format!("ndarray-{}", height), 100, f)
}

fn b_mbvecstate_init(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();

    let f = move || {
        MbVecState::initialize(width, height, &grid);
    };
    B::iter(&format!("vec-{}", height), 100, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_mbstate_init(500),
        b_mbvecstate_init(500),
        b_mbstate_init(1000),
        b_mbvecstate_init(1000),
        b_mbstate_init(2000),
        b_mbvecstate_init(2000),
    ])
    .report("stateinit");
}
