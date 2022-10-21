use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::solver::vecuv::VecUvState;

use mandelox::coord::Viewbox;
use mandelox::solver::{MbArrayState, MbVecState};

fn b_mbstate_init<T: From<Viewbox>>(name: &str, height: usize) -> Benchmark {
    let width: usize = 3 * height / 2;
    let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());

    let f = move || {
        let _state: T = v.into();
    };
    Benchmark::iter(&format!("{}-{}", name, height), 100, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_mbstate_init::<MbArrayState>("ndarray", 2000),
        b_mbstate_init::<MbVecState>("vec", 2000),
        b_mbstate_init::<VecUvState>("vecuv", 2000),
    ])
    .report("stateinit");
}
