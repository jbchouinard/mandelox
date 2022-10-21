use std::collections::HashSet;

use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::Viewbox;
use mandelox::solver::{
    MbArraySolver, MbArrayState, MbState, MbVecSolver, MbVecState, Solver, VecUvSolver, VecUvState,
};
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
    let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());
    let f = move || {
        let initial = v.into();
        solver.call(initial);
    };
    Benchmark::iter(&format!("solver-{}-{}", name, height), repeats, f)
}

fn benchmark_solver_1t<S, T>(name: &str, height: usize, repeats: usize) -> Benchmark
where
    T: MbState + 'static + Clone,
    S: Solver<T> + 'static + Default,
{
    let width: usize = (3 * height) / 2;
    let v = Viewbox::initial(width.try_into().unwrap(), height.try_into().unwrap());
    let solver = S::default();
    let initial: T = v.into();
    let f = move || {
        solver.solve(initial.clone());
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
    BenchmarkReport::with_benches(&[
        benchmark_solver_1t::<MbVecSolver, MbVecState>("vec", 500, 1),
        benchmark_solver_1t::<VecUvSolver, VecUvState>("vecuv", 500, 1),
        benchmark_solver_1t::<MbArraySolver, MbArrayState>("arr", 500, 1),
        benchmark_solver_1t::<MbVecSolver, MbVecState>("vec", 1000, 1),
        benchmark_solver_1t::<VecUvSolver, VecUvState>("vecuv", 1000, 1),
        benchmark_solver_1t::<MbArraySolver, MbArrayState>("arr", 1000, 1),
        benchmark_solver_1t::<MbVecSolver, MbVecState>("vec", 2000, 1),
        benchmark_solver_1t::<VecUvSolver, VecUvState>("vecuv", 2000, 1),
        benchmark_solver_1t::<MbArraySolver, MbArrayState>("arr", 2000, 1),
        benchmark_solver_1t::<MbVecSolver, MbVecState>("vec", 4000, 1),
        benchmark_solver_1t::<VecUvSolver, VecUvState>("vecuv", 4000, 1),
        benchmark_solver_1t::<MbArraySolver, MbArrayState>("arr", 4000, 1),
    ])
    .report("solver");
}
