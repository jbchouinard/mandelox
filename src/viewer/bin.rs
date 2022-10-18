use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use druid::widget::{Align, Button, Container, Flex, Label, Painter};
use druid::{
    AppLauncher, Color, Data, Env, Lens, PaintCtx, PlatformError, RenderContext, Size, UnitPoint,
    Widget, WidgetExt, WindowDesc,
};

use mandelox::coord::{Axis, Viewport};
use mandelox::painter::{convert_image, Painter as MandeloxPainter, RainbowPainter};
use mandelox::state::solver::MbArraySolver;
use mandelox::state::MbArrayState;
use mandelox::threads::Solver;
use mandelox::updater::Updater;

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

#[derive(Clone, Debug, Data)]
struct MbViewerState {
    width: Arc<AtomicUsize>,
    height: Arc<AtomicUsize>,
    state: Option<MbArrayState>,
}

impl MbViewerState {
    pub fn new(width: usize, height: usize, state: Option<MbArrayState>) -> Self {
        Self {
            width: Arc::new(AtomicUsize::new(width)),
            height: Arc::new(AtomicUsize::new(height)),
            state,
        }
    }

    pub fn width(&self) -> usize {
        self.width.load(Ordering::SeqCst)
    }

    pub fn height(&self) -> usize {
        self.height.load(Ordering::SeqCst)
    }

    pub fn set_width(&self, w: usize) {
        self.width.store(w, Ordering::SeqCst)
    }

    pub fn set_height(&self, h: usize) {
        self.height.store(h, Ordering::SeqCst)
    }
}

impl Default for MbViewerState {
    fn default() -> Self {
        Self::new(0, 0, None)
    }
}

#[derive(Clone, Data, Lens, Debug)]
struct ViewerState {
    viewport: Viewport,
    viewer_state: MbViewerState,
}

pub struct MbUpdater;

impl Updater<Viewport, MbViewerState> for MbUpdater {
    fn update(&mut self, old_a: &Viewport, old_b: &MbViewerState) -> MbViewerState {
        let width = old_b.width();
        let height = old_b.height();
        if width == 0 || height == 0 {
            old_b.clone()
        } else {
            let initial = MbArrayState::initialize(width, height, old_a);
            let solver = MbArraySolver::default();
            let solved = solver.solve(&initial);
            MbViewerState::new(width, height, Some(solved))
        }
    }
}

fn build_mb_painter() -> Painter<MbViewerState> {
    Painter::new(|ctx: &mut PaintCtx, data: &MbViewerState, _env: &Env| {
        let Size { width, height } = ctx.size();
        let width = width as usize;
        let height = height as usize;
        data.set_width(width);
        data.set_height(height);
        if let Some(state) = &data.state {
            let painter = RainbowPainter::new(100.0);
            let image = painter.paint(state.i_values());

            ctx.with_save(|ctx| {
                let imagebuf = convert_image(image);
                let image = imagebuf.to_image(ctx.render_ctx);
                ctx.draw_image(
                    &image,
                    imagebuf.size().to_rect(),
                    druid::piet::InterpolationMode::Bilinear,
                )
            })
        }
    })
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

// fn build_mandelbrot_painter() -> Painter<Viewport> {
//     Painter::new(|ctx: &mut PaintCtx, data: &Viewport, _env: &Env| {
//         let Size { width, height } = ctx.size();

//         let initial = MbState::initialize(width as usize, height as usize, data);
//         let solver = ThreadedMbSolver::new(2.0, 4);
//         let solved = solver.iterate_n(&initial, 100);
//         let painter = RainbowPainter::new(100.0);
//         let image = painter.paint(solved.i_values());

//         ctx.with_save(|ctx| {
//             let imagebuf = convert_image(image);
//             let image = imagebuf.to_image(ctx.render_ctx);
//             ctx.draw_image(
//                 &image,
//                 imagebuf.size().to_rect(),
//                 druid::piet::InterpolationMode::Bilinear,
//             )
//         })
//     })
// }

fn build_viewport_buttons() -> impl Widget<Viewport> {
    Container::new(
        Flex::column()
            .with_flex_child(
                Flex::row()
                    .with_child(build_zoom_button("-", 1.5))
                    .with_child(build_zoom_button("+", 1.0 / 1.5)),
                1.0,
            )
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
            ),
    )
    .padding(10.0)
}

fn build_viewport_info() -> Label<Viewport> {
    Label::new(|data: &Viewport, _env: &Env| {
        format!(
            "X {:.24}  Y {:.24}  L {:.24}",
            data.x.center(),
            data.y.center(),
            data.x.length()
        )
    })
}

fn build_ui() -> impl Widget<ViewerState> {
    // let mandelbrot_widget = build_mandelbrot_painter()
    //     .lens(ViewerState::viewport)
    //     .fix_size(1200.0, 960.0);

    let mandelbrot_widget = MbUpdater
        .async_wrapper()
        .controller(
            build_mb_painter(),
            ViewerState::viewport,
            ViewerState::viewer_state,
        )
        .fix_size(1200.0, 960.0);

    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Align::new(UnitPoint::RIGHT, mandelbrot_widget))
                .with_child(Align::new(
                    UnitPoint::LEFT,
                    build_viewport_buttons()
                        .lens(ViewerState::viewport)
                        .padding(20.0)
                        .fix_size(3.0 * NAV_BUTTON_W + 50.0, 960.0)
                        .background(Color::rgb8(0x10, 0x10, 0x10)),
                )),
        )
        .with_flex_child(
            Align::new(
                UnitPoint::TOP,
                build_viewport_info()
                    .with_text_color(Color::rgb8(0xdd, 0xdd, 0xdd))
                    .lens(ViewerState::viewport),
            ),
            1.0,
        )
        .padding(20.0)
        .background(Color::rgb8(0x10, 0x10, 0x10))
}

fn main() -> Result<(), PlatformError> {
    let initial_state = ViewerState {
        viewport: DEFAULT_VIEWPORT.clone(),
        viewer_state: MbViewerState::default(),
    };

    AppLauncher::with_window(
        WindowDesc::new(build_ui())
            .title("Mandelox")
            .window_size((1440.0, 1050.0)),
    )
    .launch(initial_state)?;
    Ok(())
}
