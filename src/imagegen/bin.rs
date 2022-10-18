use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{Painter, RainbowPainter};
use mandelox::state::{solver::MbArraySolver, MbArrayState};
use mandelox::threads::Solver;

fn main() {
    let width: usize = 1000;
    let height: usize = 800;
    let scale = Viewport::new(Axis::new(-2.0, 1.0), Axis::new(-1.2, 1.2));

    let solver = MbArraySolver::default();
    let initial = MbArrayState::initialize(width, height, &scale);
    let solved = solver.solve(&initial);
    let painter = RainbowPainter::new(10.0);
    let img = painter.paint(solved.i_values());
    img.save("out.png").expect("failed to save image");
}
