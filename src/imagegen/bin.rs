use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{Painter, RainbowPainter};
use mandelox::{MandelbrotSolver, MandelbrotState};

fn main() {
    let width: usize = 1500;
    let height: usize = 1200;
    let scale = Viewport::new(Axis::new(-2.0, 1.0), Axis::new(-1.2, 1.2));

    let initial = MandelbrotState::initialize(width, height, &scale);
    let solver = MandelbrotSolver::new(2.0);
    let solved = solver.iterate_n(&initial, 100);
    let painter = RainbowPainter::new(100.0);
    let img = painter.paint(solved.i_values());
    img.save("out.png").expect("failed to save image");
}
