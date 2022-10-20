use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{IValuePainter, Painter, Rainbow};
use mandelox::solver::{default_solver, MbState, MbVecState};
use mandelox::threads::Call;

fn main() {
    let width: usize = 2000;
    let height: usize = 1600;
    let scale = Viewport::new(Axis::new(-2.0, 1.0), Axis::new(-1.2, 1.2));

    let solver = default_solver();
    let initial = MbVecState::initialize(width, height, &scale);

    let solved = solver.call(initial);
    let painter = IValuePainter::new(Rainbow, 100);
    let img = painter.paint(&solved);
    img.save("out.png").expect("failed to save image");
}
