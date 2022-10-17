use mandelox::bench::{Benchmark as B, BenchmarkReport};
use mandelox::coord::Viewport;
use mandelox::state::solver::{IterSolver, MbSolver, VecIterSolver};
use mandelox::state::{MbState, MbVecState};

fn b_mbstate_init(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();

    let f = move || {
        MbState::initialize(width, height, &grid);
    };
    B::iter(&format!("ndarray-init-{}", height), 100, f)
}

fn b_mbvecstate_init(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let grid = Viewport::default();

    let f = move || {
        MbVecState::initialize(width, height, &grid);
    };
    B::iter(&format!("vec-init-{}", height), 100, f)
}

fn b_itersolve_1t(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let solver = IterSolver::default();
    let grid = Viewport::default();
    let initial = MbState::initialize(width, height, &grid);

    let f = move || {
        solver.solve(&initial);
    };
    B::iter(&format!("ndarray-solve-1t-{}", height), 3, f)
}

fn b_vecitersolve_1t(height: usize) -> B {
    let width: usize = 3 * height / 2;
    let solver = VecIterSolver::default();
    let grid = Viewport::default();
    let initial = MbVecState::initialize(width, height, &grid);

    let f = move || {
        solver.solve(&initial);
    };
    B::iter(&format!("vec-solve-1t-{}", height), 3, f)
}

fn main() {
    let mut report = BenchmarkReport::with_benches(&[
        b_mbstate_init(500),
        b_mbvecstate_init(500),
        b_mbstate_init(1000),
        b_mbvecstate_init(1000),
        b_mbstate_init(2000),
        b_mbvecstate_init(2000),
        b_itersolve_1t(500),
        b_vecitersolve_1t(500),
        b_itersolve_1t(1000),
        b_vecitersolve_1t(1000),
        b_itersolve_1t(2000),
        b_vecitersolve_1t(2000),
    ]);
    report.run();
    report.show();
    report.write_csv("benchmark_solver.csv");
}
