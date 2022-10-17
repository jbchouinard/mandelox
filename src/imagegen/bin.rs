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

// New threads spawned on each frame

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

// 1500x200,100 amd r9-5900x 12c/24t
// t=1 1.170 1.157 1.197
// t=2 0.805 0.795 0.764
// t=3 0.681 0.657 0.658
// t=4 0.621 0.626 0.625
// t=6 0.631 0.620 0.616
// t=8 0.643 0.603 0.600
// t=12 0.661 0.608 0.617
// t=24 0.674 0.672 0.673

// 3000x2400,100 amd r9-5900x 12c/24t
// t=1 4.213 4.242 4.224
// t=2 3.010 2.930 2.977
// t=3 2.653 2.637 2.654
// t=4 2.369 2.383 2.403
// t=6 2.388 2.401 2.454
// t=8 2.397 2.400 2.407
// t=12 2.490 2.507 2.477
// t=24 2.649 2.649 2.667

// Persistent workers
