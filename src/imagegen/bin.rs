use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{Painter, RainbowPainter};
use mandelox::{IterSolver, MbSolver, MbState, MultiSolver};

fn main() {
    let width: usize = 3000;
    let height: usize = 2400;
    let scale = Viewport::new(Axis::new(-2.0, 1.0), Axis::new(-1.2, 1.2));

    let initial = MbState::initialize(width, height, &scale);
    let solver = MultiSolver::with_cloned_solvers(1, &IterSolver::new(2.0, 100));
    let solved = solver.solve(&initial);
    let painter = RainbowPainter::new(10.0);
    let img = painter.paint(solved.i_values());
    img.save("out.png").expect("failed to save image");
}
