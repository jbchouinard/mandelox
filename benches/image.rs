use mandelox::bench::Benchmark;
use mandelox::bench::BenchmarkReport;
use mandelox::coord::Viewbox;
use mandelox::painter::Greyscale;
use mandelox::painter::IValuePainter;
use mandelox::painter::Painter;
use mandelox::solver::MbState;
use mandelox::solver::Solver;
use mandelox::solver::VecSolver;
use mandelox::threads::Call;
use mandelox::threads::Join;
use mandelox::threads::Split;

const REPEATS: usize = 10;

fn generate_image<S, T>(threads: usize, width: usize, height: usize, paint: bool)
where
    T: MbState + Split + Join + Send + 'static,
    S: Solver<T> + Default + Clone + Send + 'static,
{
    let scale = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());

    let solver = S::default().threaded(threads);
    let initial = scale.generate_complex_coordinates().into();
    let solved = solver.call(initial);

    if paint {
        let painter = IValuePainter::new(Greyscale, 100);
        painter.paint(&solved);
    }
}

fn benchmark_image<S, T>(threads: usize, size: usize, paint: bool) -> Benchmark
where
    S: Solver<T> + Default + Clone + Send + 'static,
    T: MbState + Split + Join + Send + 'static,
{
    let name = &format!("image t={} r={}x{} p={}", threads, size, size, paint);
    let f = move || generate_image::<S, T>(threads, size, size, paint);
    Benchmark::iter(name, REPEATS, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        benchmark_image::<VecSolver, _>(1, 500, true),
        benchmark_image::<VecSolver, _>(2, 500, true),
        benchmark_image::<VecSolver, _>(4, 500, true),
        benchmark_image::<VecSolver, _>(8, 500, true),
        benchmark_image::<VecSolver, _>(1, 1000, true),
        benchmark_image::<VecSolver, _>(2, 1000, true),
        benchmark_image::<VecSolver, _>(4, 1000, true),
        benchmark_image::<VecSolver, _>(8, 1000, true),
        benchmark_image::<VecSolver, _>(1, 2000, true),
        benchmark_image::<VecSolver, _>(2, 2000, true),
        benchmark_image::<VecSolver, _>(4, 2000, true),
        benchmark_image::<VecSolver, _>(8, 2000, true),
    ])
    .report("image");
}
