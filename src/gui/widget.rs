use druid::widget::prelude::*;
use druid::{Code, MouseButton, Widget};

use crate::gui::convert_image;
use crate::MandelbrotWorker;

pub struct MandelbrotWidget {
    worker: MandelbrotWorker,
    width: i64,
    height: i64,
}

impl MandelbrotWidget {
    pub fn new() -> Self {
        Self {
            worker: MandelbrotWorker::new(),
            width: 0,
            height: 0,
        }
    }
}

impl MandelbrotWidget {
    fn resize(&mut self, size: Size) -> bool {
        let height = f64::round(size.height) as i64;
        let width = f64::round(size.width) as i64;
        if !(self.width == width && self.height == height) {
            self.worker.resize(width, height);
            self.width = width;
            self.height = height;
            true
        } else {
            false
        }
    }
}

impl Widget<()> for MandelbrotWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        if self.worker.images_count() > 0 {
            ctx.request_paint();
        }
        match event {
            Event::KeyDown(key_event) => {
                use Code::*;
                match key_event.code {
                    ArrowUp => self.worker.pan_relative(0.0, -0.05),
                    ArrowDown => self.worker.pan_relative(0.0, 0.05),
                    ArrowLeft => self.worker.pan_relative(-0.05, 0.0),
                    ArrowRight => self.worker.pan_relative(0.05, 0.0),
                    PageUp => self.worker.zoom(1.2),
                    PageDown => self.worker.zoom(1.0 / 1.2),
                    KeyR => self.worker.reset(self.width, self.height),
                    _ => (),
                }
            }
            Event::MouseMove(_) => {
                if !ctx.is_focused() {
                    ctx.request_focus();
                }
                // TODO: drag-and-drop movement
            }
            Event::MouseDown(mouse) => {
                if let MouseButton::Left = mouse.button {
                    let druid::Point { x, y } = mouse.pos;
                    let x = f64::round(x) as i64;
                    let y = f64::round(y) as i64;
                    self.worker.pan(x - (self.width / 2), y - (self.height / 2));
                }
            }
            Event::Wheel(mouse) => {
                let delta_y = mouse.wheel_delta.y;
                let zf = if delta_y > 0.0 {
                    1.0 / (1.0 + delta_y / 1000.0)
                } else {
                    1.0 + delta_y / -1000.0
                };
                self.worker.zoom(zf);
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &(), _env: &Env) {
        match event {
            LifeCycle::WidgetAdded => {
                ctx.register_for_focus();
                self.resize(ctx.size());
            }
            LifeCycle::Size(size) => {
                self.resize(*size);
            }
            _ => (),
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &(), _new_data: &(), _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &(),
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &(), _env: &Env) {
        let size = ctx.size();
        if !self.resize(size) {
            if let Some(rgb_image) = self.worker.next_image() {
                let image_buf = convert_image(rgb_image);
                let ctx_image = image_buf.to_image(ctx.render_ctx);
                ctx.draw_image(
                    &ctx_image,
                    size.to_rect(),
                    druid::piet::InterpolationMode::NearestNeighbor,
                );
            }
        }
    }
}
