use std::collections::HashSet;

use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::solver::{MbArraySolver, MbState, MbVecSolver, Solver};
use mandelox::threads::Call;

fn thread_counts() -> Vec<usize> {
    let cpus = num_cpus::get_physical();
    let threads = num_cpus::get();
    let mut tcounts: HashSet<usize> = HashSet::new();

    tcounts.insert(1);
    tcounts.insert(2);
    tcounts.insert(4);
    tcounts.insert(cpus);
    tcounts.insert(threads);

    let mut tcounts: Vec<usize> = tcounts.into_iter().collect();
    tcounts.sort();
    tcounts
}

fn benchmark_solver<S, T, U>(name: &str, solver: S, height: usize, repeats: usize) -> Benchmark
where
    T: MbState + 'static,
    S: Call<T, U> + 'static,
{
    let width: usize = (3 * height) / 2;
    let grid = Viewport::default();
    let f = move || {
        let initial = T::initialize(width, height, &grid);
        solver.call(initial);
    };
    Benchmark::iter(&format!("solver-{}-{}", name, height), repeats, f)
}

fn benchmarks(height: usize, repeats: usize) -> Vec<Benchmark> {
    let mut benches = vec![];
    for t in thread_counts() {
        benches.push(benchmark_solver(
            &format!("arr-{}", t),
            MbArraySolver::default().threaded(t),
            height,
            repeats,
        ));
        benches.push(benchmark_solver(
            &format!("vec-{}", t),
            MbVecSolver::default().threaded(t),
            height,
            repeats,
        ));
    }
    benches
}

fn main() {
    BenchmarkReport::with_benches(&benchmarks(1000, 10)).report("solver");
}