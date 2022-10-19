use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use druid::lens::Identity;
use druid::widget::{Align, Button, Container, Controller, Flex, Label, Painter};
use druid::{
    AppLauncher, Code, Color, Data, Env, Event, EventCtx, FontDescriptor, FontFamily, FontWeight,
    Lens, PaintCtx, PlatformError, RenderContext, Size, UnitPoint, Widget, WidgetExt, WindowDesc,
};

use mandelox::coord::Viewport;
use mandelox::painter::{convert_image, IValuePainter, Painter as MandeloxPainter, Rainbow};
use mandelox::solver::{MbState, MbVecSolver, MbVecState, Solver};
use mandelox::threads::Call;
use mandelox::updater::{Refresher, Updater};

const VIEWER_W: f64 = 1200.0;
const VIEWER_H: f64 = 800.0;
const NAV_BUTTON_W: f64 = 40.0;
const NAV_BUTTON_H: f64 = 40.0;

fn default_viewport() -> Viewport {
    let ratio = VIEWER_W / VIEWER_H;
    Viewport::from_box(-0.5, 0.0, 3.0, 3.0 / ratio)
}

#[derive(Clone, Data, Lens, Debug)]
struct AppState {
    width: Arc<AtomicUsize>,
    height: Arc<AtomicUsize>,
    state: Option<MbVecState>,
    viewport: Viewport,
}

impl AppState {
    pub fn new(width: usize, height: usize, state: Option<MbVecState>, viewport: Viewport) -> Self {
        Self {
            width: Arc::new(AtomicUsize::new(width)),
            height: Arc::new(AtomicUsize::new(height)),
            state,
            viewport,
        }
    }

    pub fn get_width(&self) -> usize {
        self.width.load(Ordering::SeqCst)
    }

    pub fn get_height(&self) -> usize {
        self.height.load(Ordering::SeqCst)
    }

    pub fn set_width(&self, w: usize) {
        self.width.store(w, Ordering::SeqCst)
    }

    pub fn set_height(&self, h: usize) {
        self.height.store(h, Ordering::SeqCst)
    }

    pub fn initial() -> Self {
        Self::new(0, 0, None, default_viewport())
    }
}

pub struct MbUpdater;

impl Updater<Viewport, AppState> for MbUpdater {
    fn update(&mut self, old_a: &Viewport, old_b: &AppState) -> AppState {
        let width = old_b.get_width();
        let height = old_b.get_height();
        if width == 0 || height == 0 {
            old_b.clone()
        } else {
            let initial = MbVecState::initialize(width, height, old_a);
            let solver = MbVecSolver::default().threaded(num_cpus::get_physical());
            let solved = solver.call(initial);
            // let solver = MbVecSolver::default();
            // let solved = solver.solve(initial);
            AppState::new(width, height, Some(solved), old_a.clone())
        }
    }
}

fn build_mb_painter() -> Painter<AppState> {
    Painter::new(|ctx: &mut PaintCtx, data: &AppState, _env: &Env| {
        let Size { width, height } = ctx.size();
        let width = width as usize;
        let height = height as usize;
        data.set_width(width);
        data.set_height(height);
        if let Some(state) = &data.state {
            let painter = IValuePainter::new(Rainbow, 100);
            let image = painter.paint(state);

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

struct ViewportControl;

impl<W> Controller<AppState, W> for ViewportControl
where
    W: Widget<AppState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        if !ctx.has_focus() {
            ctx.request_focus();
        }
        println!("{:?}", event);
        match event {
            Event::KeyDown(key_event) => match key_event.code {
                Code::ArrowUp => data.viewport.pan_relative(0.0, -0.05),
                Code::ArrowDown => data.viewport.pan_relative(0.0, 0.05),
                Code::ArrowLeft => data.viewport.pan_relative(-0.05, 0.00),
                Code::ArrowRight => data.viewport.pan_relative(0.05, 0.00),
                Code::PageUp => data.viewport.zoom(1.0 / 1.2),
                Code::PageDown => data.viewport.zoom(1.2),
                _ => child.event(ctx, event, data, env),
            },
            _ => child.event(ctx, event, data, env),
        }
    }
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
        .on_click(move |_ctx, data: &mut Viewport, _env| *data = default_viewport())
        .fix_size(NAV_BUTTON_W, NAV_BUTTON_H)
}

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
                    .with_child(build_pan_button("▲ ", 0.0, -0.10))
                    .with_spacer(NAV_BUTTON_W),
                1.0,
            )
            .with_flex_child(
                Flex::row()
                    .with_child(build_pan_button("◀", -0.10, 0.0))
                    .with_child(build_reset_button("R"))
                    .with_child(build_pan_button("▶", 0.10, 0.0)),
                1.0,
            )
            .with_flex_child(
                Flex::row()
                    .with_spacer(NAV_BUTTON_W)
                    .with_child(build_pan_button(" ▼ ", 0.0, 0.10))
                    .with_spacer(NAV_BUTTON_W),
                1.0,
            ),
    )
    .padding(10.0)
}

fn flabel<F: Fn(&T) -> f64 + 'static, T: Data>(name: &str, f: F) -> Label<T> {
    let name = name.to_string();
    Label::new(move |data: &T, _: &Env| format!("{} {:+1.16}", name, f(data)))
        .with_text_color(Color::rgb8(0x88, 0x88, 0x88))
        .with_font(
            FontDescriptor::new(FontFamily::MONOSPACE)
                .with_size(11.0)
                .with_weight(FontWeight::SEMI_BOLD),
        )
}

fn build_viewport_info() -> impl Widget<Viewport> {
    Flex::column()
        .with_flex_child(flabel("X", |data: &Viewport| data.x.center()), 1.0)
        .with_flex_child(flabel("Y", |data: &Viewport| data.y.center()), 1.0)
        .with_flex_child(flabel("L", |data: &Viewport| data.x.length()), 1.0)
        .padding(10.0)
}

fn build_mandelbrot() -> impl Widget<AppState> {
    MbUpdater
        .async_wrapper()
        .controller(build_mb_painter(), AppState::viewport, Identity)
        .controller(Refresher::new(100, true))
        .fix_size(VIEWER_W, VIEWER_H)
        .controller(ViewportControl)
}

#[allow(dead_code)]
fn build_ui_with_nav() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Align::new(UnitPoint::RIGHT, build_mandelbrot()))
                .with_child(Align::new(
                    UnitPoint::LEFT,
                    build_viewport_buttons()
                        .lens(AppState::viewport)
                        .padding(10.0)
                        .fix_size(3.0 * NAV_BUTTON_W + 50.0, VIEWER_H)
                        .background(Color::rgb8(0x10, 0x10, 0x10)),
                )),
        )
        .with_flex_child(
            Align::new(
                UnitPoint::BOTTOM_LEFT,
                build_viewport_info().lens(AppState::viewport),
            ),
            1.0,
        )
        .padding(10.0)
        .background(Color::rgb8(0x10, 0x10, 0x10))
}

fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(Align::new(UnitPoint::RIGHT, build_mandelbrot()))
        .with_flex_child(
            Align::new(
                UnitPoint::BOTTOM_RIGHT,
                build_viewport_info().lens(AppState::viewport),
            ),
            1.0,
        )
        .padding(10.0)
        .background(Color::rgb8(0x10, 0x10, 0x10))
}

fn main() -> Result<(), PlatformError> {
    let initial_state = AppState::initial();

    AppLauncher::with_window(
        WindowDesc::new(build_ui())
            .title("Mandelox")
            .window_size((VIEWER_W + 20.0, VIEWER_H + 70.0)),
    )
    .launch(initial_state)?;
    Ok(())
}
