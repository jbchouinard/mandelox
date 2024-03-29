use num::complex::Complex;

use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::{Coords, Viewbox};
use mandelox::solver::simdvec::SimdVecState;
use mandelox::solver::{ArrayState, VecState};

fn b_mbstate_init<T: From<Coords<Complex<f64>>>>(name: &str, height: usize) -> Benchmark {
    let width: usize = 3 * height / 2;
    let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());

    let f = move || {
        let _state: T = v.generate_complex_coordinates().into();
    };
    Benchmark::iter(&format!("{}-{}", name, height), 100, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_mbstate_init::<ArrayState>("ndarray", 2000),
        b_mbstate_init::<VecState>("vec", 2000),
        b_mbstate_init::<SimdVecState>("vecuv", 2000),
    ])
    .report("stateinit");
}
