use std::collections::HashSet;

use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::Viewbox;
use mandelox::solver::{ArraySolver, MbState, SimdVecSolver, Solver, VecSolver};
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
    T: MbState + 'static + Clone,
    S: Call<T, U> + 'static,
{
    let width: usize = (3 * height) / 2;
    let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());
    let initial: T = v.generate_complex_coordinates().into();
    let f = move || {
        solver.call(initial.clone());
    };
    Benchmark::iter(&format!("{}  {:>4}", name, height), repeats, f)
}

// fn benchmark_solver_1t<S, T>(name: &str, height: usize, repeats: usize) -> Benchmark
// where
//     T: MbState + 'static + Clone,
//     S: Solver<T> + 'static + Default,
// {
//     let width: usize = (3 * height) / 2;
//     let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());
//     let solver = S::default();
//     let initial: T = v.into();
//     let f = move || {
//         solver.solve(initial.clone());
//     };
//     Benchmark::iter(&format!("solver-{}-{}", name, height), repeats, f)
// }

fn benchmarks(height: usize, repeats: usize) -> Vec<Benchmark> {
    let mut benches = vec![];
    for t in thread_counts() {
        benches.push(benchmark_solver(
            &format!("arr      {:>2}t", t),
            ArraySolver::default().threaded(t),
            height,
            repeats,
        ));
        benches.push(benchmark_solver(
            &format!("vec      {:>2}t", t),
            VecSolver::default().threaded(t),
            height,
            repeats,
        ));
        benches.push(benchmark_solver(
            &format!("simdvec  {:>2}t", t),
            SimdVecSolver::default().threaded(t),
            height,
            repeats,
        ));
    }
    benches
}

fn main() {
    BenchmarkReport::with_benches(&benchmarks(1000, 10)).report("solver");
}
