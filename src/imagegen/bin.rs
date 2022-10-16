use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{Painter, RainbowPainter};
use mandelox::{IterSolver, MbState, ThreadedMbSolver};

fn main() {
    let width: usize = 3000;
    let height: usize = 2400;
    let scale = Viewport::new(Axis::new(-2.0, 1.0), Axis::new(-1.2, 1.2));

    let initial = MbState::initialize(width, height, &scale);
    let solver = ThreadedMbSolver::new(2.0, 1);
    let solved = solver.iterate_n(&initial, 100);
    let painter = RainbowPainter::new(100.0);
    let img = painter.paint(solved.i_values());
    img.save("out.png").expect("failed to save image");
}

// 1500x1200,100 intel i7-1165G7 4c/8t
// t=1 1.280 1.546 1.302 1.295 1.299
// t=2 0.907 0.928 0.918 0.947 0.933
// t=4 0.794 0.788 0.789 0.780 0.791
// t=8 0.793 0.805 0.801 0.808 0.802

// 3000x2400,100 intel i7-1165G7 4c/8t
// t=1 7.977 8.299 8.513
// t=2 6.110 6.012 5.370
// t=4 3.140 3.448 3.563
// t=8 3.432 3.085 3.404
