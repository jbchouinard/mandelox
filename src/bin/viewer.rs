use druid::{AppLauncher, PlatformError, WindowDesc};

use mandelox::gui::widget::MandelbrotWidget;

fn main() -> Result<(), PlatformError> {
    AppLauncher::with_window(
        WindowDesc::new(MandelbrotWidget::new())
            .title("Mandelox")
            .window_size((1200.0, 1200.0)),
    )
    .launch(())?;
    Ok(())
}
