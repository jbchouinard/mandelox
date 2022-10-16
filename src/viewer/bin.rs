use druid::widget::{Align, Button, Container, Flex, Label, Painter};
use druid::{
    AppLauncher, Color, Data, Env, Lens, PaintCtx, PlatformError, RenderContext, Size, UnitPoint,
    Widget, WidgetExt, WindowDesc,
};

use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{convert_image, Painter as MandeloxPainter, RainbowPainter};
use mandelox::{IterSolver, MbSolver, MbState};

const DEFAULT_VIEWPORT: Viewport = Viewport {
    x: Axis {
        min: -2.0,
        max: 1.0,
    },
    y: Axis {
        min: -1.2,
        max: 1.2,
    },
};

#[derive(Clone, Data, Lens)]
struct ViewerState {
    viewport: Viewport,
}

fn build_pan_button(text: &str, x: f64, y: f64) -> impl Widget<Viewport> {
    Button::new(text)
        .on_click(move |_ctx, data: &mut Viewport, _env| data.pan_relative(x, y))
        .fix_size(NAV_BUTTON_W, NAV_BUTTON_H)
}

fn build_zoom_button(text: &str, factor: f64) -> impl Widget<Viewport> {
    Button::new(text)
        .on_click(move |_ctx, data: &mut Viewport, _env| data.zoom(factor))
        .fix_size(1.5 * NAV_BUTTON_W, NAV_BUTTON_H)
}

fn build_reset_button(text: &str) -> impl Widget<Viewport> {
    Button::new(text)
        .on_click(move |_ctx, data: &mut Viewport, _env| *data = DEFAULT_VIEWPORT.clone())
        .fix_size(NAV_BUTTON_W, NAV_BUTTON_H)
}

const NAV_BUTTON_W: f64 = 40.0;
const NAV_BUTTON_H: f64 = 40.0;

fn build_mandelbrot_painter() -> Painter<Viewport> {
    Painter::new(|ctx: &mut PaintCtx, data: &Viewport, _env: &Env| {
        let Size { width, height } = ctx.size();

        let initial = MbState::initialize(width as usize, height as usize, data);
        let solver = MbSolver::new(2.0);
        let solved = solver.iterate_n(&initial, 100);
        let painter = RainbowPainter::new(100.0);
        let image = painter.paint(solved.i_values());

        ctx.with_save(|ctx| {
            let imagebuf = convert_image(image);
            let image = imagebuf.to_image(ctx.render_ctx);
            ctx.draw_image(
                &image,
                imagebuf.size().to_rect(),
                druid::piet::InterpolationMode::Bilinear,
            )
        })
    })
}

fn build_viewport_buttons() -> impl Widget<Viewport> {
    Container::new(
        Flex::column()
            .with_flex_child(
                Flex::row()
                    .with_spacer(NAV_BUTTON_W)
                    .with_child(build_pan_button("⮝", 0.0, -0.10))
                    .with_spacer(NAV_BUTTON_W),
                1.0,
            )
            .with_flex_child(
                Flex::row()
                    .with_child(build_pan_button("⮜", -0.10, 0.0))
                    .with_child(build_reset_button("R"))
                    .with_child(build_pan_button("⮞", 0.10, 0.0)),
                1.0,
            )
            .with_flex_child(
                Flex::row()
                    .with_spacer(NAV_BUTTON_W)
                    .with_child(build_pan_button("⮟", 0.0, 0.10))
                    .with_spacer(NAV_BUTTON_W),
                1.0,
            )
            .with_flex_child(
                Flex::row()
                    .with_child(build_zoom_button("-", 1.5))
                    .with_child(build_zoom_button("+", 1.0 / 1.5)),
                1.0,
            ),
    )
    .padding(10.0)
    .fix_size(3.0 * NAV_BUTTON_W + 10.0, 4.0 * NAV_BUTTON_H + 20.0)
}

fn build_viewport_info() -> impl Widget<Viewport> {
    Label::new(|data: &Viewport, _env: &Env| {
        format!(
            "X {:.20}\nY {:.20}\nL {:.20}",
            data.x.center(),
            data.y.center(),
            data.x.length()
        )
    })
    .with_text_color(Color::rgb8(12, 12, 12))
}

fn build_viewport_navigation() -> impl Widget<Viewport> {
    Flex::row()
        .with_flex_child(
            Align::new(UnitPoint::TOP_RIGHT, build_viewport_buttons()),
            1.0,
        )
        .with_flex_child(Align::new(UnitPoint::TOP_RIGHT, build_viewport_info()), 1.0)
        .padding(10.0)
        .background(Color::rgb8(180, 180, 180))
        .rounded(10.0)
        .fix_height(4.0 * NAV_BUTTON_H + 30.0)
}

fn build_ui() -> impl Widget<ViewerState> {
    Flex::row()
        .with_flex_spacer(1.0)
        .with_child(Align::new(
            UnitPoint::TOP_RIGHT,
            build_mandelbrot_painter()
                .lens(ViewerState::viewport)
                .fix_size(1200.0, 960.0),
        ))
        .with_flex_child(
            Align::new(
                UnitPoint::TOP_LEFT,
                build_viewport_navigation()
                    .lens(ViewerState::viewport)
                    .padding(20.0),
            ),
            1.0,
        )
        .padding(40.0)
}

fn main() -> Result<(), PlatformError> {
    let initial_state = ViewerState {
        viewport: DEFAULT_VIEWPORT.clone(),
    };

    AppLauncher::with_window(WindowDesc::new(build_ui()).title("Mandelox v0.1.0"))
        .launch(initial_state)?;
    Ok(())
}
