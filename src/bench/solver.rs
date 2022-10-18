use mandelox::bench::{Benchmark as B, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::state::solver::MbArraySolver;
use mandelox::state::MbState;
use mandelox::threads::{DefaultThreaded, Solver, ThreadedSolver};

static HEIGHT: usize = 2000;
static REPEATS: usize = 10;

fn b_solver<S, T>(name: &str, solver: S, height: usize) -> B
where
    T: MbState + 'static,
    S: Solver<T> + 'static,
{
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();
    let initial = T::initialize(width, height, &grid);
    let f = move || {
        solver.solve(&initial);
    };
    B::iter(&format!("solver-{}-{}", name, height), REPEATS, f)
}

fn main() {
    BenchmarkReport::with_benches(&[
        b_solver("arr-st", MbArraySolver::default(), HEIGHT),
        b_solver("arr-mt2", MbArraySolver::threaded(2), HEIGHT),
        b_solver("arr-mt4", MbArraySolver::threaded(4), HEIGHT),
        b_solver(
            "arr-mt2x2",
            ThreadedSolver::with_solvers(2, || MbArraySolver::threaded(2)),
            HEIGHT,
        ),
        b_solver("arr-mt8", MbArraySolver::threaded(8), HEIGHT),
        b_solver(
            "arr-mt2x4",
            ThreadedSolver::with_solvers(2, || MbArraySolver::threaded(4)),
            HEIGHT,
        ),
        b_solver(
            "arr-mt4x2",
            ThreadedSolver::with_solvers(4, || MbArraySolver::threaded(2)),
            HEIGHT,
        ),
        b_solver(
            "arr-mt2x2x2",
            ThreadedSolver::with_solvers(2, || {
                ThreadedSolver::with_solvers(2, || MbArraySolver::threaded(2))
            }),
            HEIGHT,
        ),
        b_solver("arr-mt16", MbArraySolver::threaded(16), HEIGHT),
        b_solver(
            "arr-mt4x4",
            ThreadedSolver::with_solvers(4, || MbArraySolver::threaded(4)),
            HEIGHT,
        ),
        b_solver(
            "arr-mt2x8",
            ThreadedSolver::with_solvers(2, || MbArraySolver::threaded(8)),
            HEIGHT,
        ),
        b_solver(
            "arr-mt8x2",
            ThreadedSolver::with_solvers(8, || MbArraySolver::threaded(2)),
            HEIGHT,
        ),
        b_solver(
            "arr-mt2x2x2x2",
            ThreadedSolver::with_solvers(2, || {
                ThreadedSolver::with_solvers(2, || {
                    ThreadedSolver::with_solvers(2, || MbArraySolver::threaded(2))
                })
            }),
            HEIGHT,
        ),
    ])
    .report("solver");
}
