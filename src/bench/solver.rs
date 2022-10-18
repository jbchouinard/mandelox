use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::state::solver::{MbArraySolver, MbVecSolver};
use mandelox::state::MbState;
use mandelox::threads::{DefaultThreaded, Solver};

static HEIGHT: usize = 1000;
static REPEATS: usize = 10;

fn b_solver<S, T>(name: &str, solver: S, height: usize) -> Benchmark
where
    T: MbState + 'static,
    S: Solver<T> + 'static,
{
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();
    let f = move || {
        let initial = T::initialize(width, height, &grid);
        solver.solve(initial);
    };
    Benchmark::iter(&format!("solver-{}-{}", name, height), REPEATS, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_solver("vec-st", MbVecSolver::default(), HEIGHT),
        b_solver("vec-mt2", MbVecSolver::threaded(2), HEIGHT),
        b_solver("vec-mt4", MbVecSolver::threaded(4), HEIGHT),
        b_solver("vec-mt8", MbVecSolver::threaded(8), HEIGHT),
        b_solver("vec-mt16", MbVecSolver::threaded(16), HEIGHT),
        b_solver("arr-st", MbArraySolver::default(), HEIGHT),
        b_solver("arr-mt2", MbArraySolver::threaded(2), HEIGHT),
        b_solver("arr-mt4", MbArraySolver::threaded(4), HEIGHT),
        b_solver("arr-mt8", MbArraySolver::threaded(8), HEIGHT),
        b_solver("arr-mt16", MbArraySolver::threaded(16), HEIGHT),
    ])
    .report("solver");
}
