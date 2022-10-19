use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::solver::{MbArrayState, MbState, MbVecState};

fn b_mbstate_init<T: MbState>(name: &str, height: usize) -> Benchmark {
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();

    let f = move || {
        T::initialize(width, height, &grid);
    };
    Benchmark::iter(&format!("{}-{}", name, height), 100, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_mbstate_init::<MbArrayState>("ndarray", 2000),
        b_mbstate_init::<MbVecState>("vec", 2000),
    ])
    .report("stateinit");
}
