use druid::{AppLauncher, PlatformError, WindowDesc};

use mandelox::gui::widget::MandelbrotWidget;

const VIEWER_W: f64 = 800.0;
const VIEWER_H: f64 = 800.0;

fn main() -> Result<(), PlatformError> {
    let initial_state = ();

    AppLauncher::with_window(
        WindowDesc::new(MandelbrotWidget::new())
            .title("Mandelox")
            .window_size((VIEWER_W, VIEWER_H)),
    )
    .launch(initial_state)?;
    Ok(())
}
