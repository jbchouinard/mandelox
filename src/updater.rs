use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use druid::widget::Controller;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, TimerToken, UpdateCtx, Widget,
};

pub struct Refresher {
    frequency: Duration,
    request_paint: bool,
    timer_token: Option<TimerToken>,
}

impl Refresher {
    pub fn new(frequency: u64, request_paint: bool) -> Self {
        Self {
            frequency: Duration::from_millis(frequency),
            request_paint,
            timer_token: None,
        }
    }
}

impl<T, W> Controller<T, W> for Refresher
where
    T: Data,
    W: Widget<T>,
{
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.timer_token = match self.timer_token {
            Some(token) => Some(token),
            None => Some(ctx.request_timer(self.frequency)),
        };
        if let Event::Timer(token) = event {
            if Some(token) == self.timer_token.as_ref() {}
            if self.request_paint {
                ctx.request_paint();
            }
            self.timer_token = Some(ctx.request_timer(self.frequency));
        }
        child.event(ctx, event, data, env)
    }
}

/// Update state of B based on A
pub trait Updater<A, B>
where
    A: Data + Send + 'static,
    B: Data + Send + 'static,
{
    fn update(&mut self, old_a: &A, old_b: &B) -> B;

    fn controller<W, T, LA, LB>(
        self,
        widget: W,
        lens_a: LA,
        lens_b: LB,
    ) -> UpdateController<W, T, Self, A, B, LA, LB>
    where
        Self: Sized,
        W: Widget<B> + 'static,
        T: Data,
        LA: Lens<T, A>,
        LB: Lens<T, B>,
    {
        UpdateController::new(self, widget, lens_a, lens_b)
    }
    fn async_wrapper(self) -> AsyncUpdateWrapper<A, B>
    where
        Self: Sized + Send + 'static,
    {
        AsyncUpdateWrapper::new(self)
    }
}

pub struct UpdateController<W, T, U, A, B, LA, LB>
where
    W: Widget<B>,
    T: Data,
    U: Updater<A, B>,
    A: Data + Send + 'static,
    B: Data + Send + 'static,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
{
    widget: W,
    updater: U,
    lens_a: LA,
    lens_b: LB,
    updated: bool,
    t: PhantomData<T>,
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<W, T, U, A, B, LA, LB> UpdateController<W, T, U, A, B, LA, LB>
where
    W: Widget<B> + 'static,
    T: Data,
    U: Updater<A, B>,
    A: Data + Send + 'static,
    B: Data + Send + 'static,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
{
    pub fn new(updater: U, widget: W, lens_a: LA, lens_b: LB) -> Self {
        Self {
            updater,
            widget,
            lens_a,
            lens_b,
            updated: true,
            t: PhantomData,
            a: PhantomData,
            b: PhantomData,
        }
    }
}

impl<W, T, U, A, B, LA, LB> Widget<T> for UpdateController<W, T, U, A, B, LA, LB>
where
    W: Widget<B> + 'static,
    T: Data,
    U: Updater<A, B>,
    A: Data + Send + 'static,
    B: Data + Send + 'static,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
{
    fn event(&mut self, ctx: &mut EventCtx, _event: &Event, data: &mut T, _env: &Env) {
        if self.updated {
            let updated_data_b: B = self.lens_a.with(data, |data_a| {
                self.lens_b
                    .with(data, |data_b| self.updater.update(data_a, data_b))
            });
            self.lens_b
                .with_mut(data, |data_b| *data_b = updated_data_b);
            self.updated = false;
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, new_data: &T, _env: &Env) {
        let old_data_a: A = self.lens_a.with(old_data, |old_data_a| old_data_a.clone());
        let new_data_a: A = self.lens_a.with(new_data, |new_data_a| new_data_a.clone());
        if !old_data_a.same(&new_data_a) {
            self.updated = true;
            ctx.request_timer(std::time::Duration::from_millis(100));
        }
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        self.lens_b.with(data, |data_b| {
            self.widget.layout(layout_ctx, bc, data_b, env)
        })
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.lens_b
            .with(data, |data_b| self.widget.paint(ctx, data_b, env))
    }
}

pub struct AsyncUpdateWrapper<A, B> {
    h: thread::JoinHandle<()>,
    tx: mpsc::Sender<(A, B)>,
    rx: mpsc::Receiver<B>,
    shutdown: Arc<AtomicBool>,
}

impl<A, B> AsyncUpdateWrapper<A, B>
where
    A: Data + Send + 'static,
    B: Data + Send + 'static,
{
    pub fn new<U>(mut updater: U) -> Self
    where
        U: Updater<A, B> + Send + 'static,
    {
        let (ab_tx, ab_rx) = mpsc::channel::<(A, B)>();
        let (b_tx, b_rx) = mpsc::channel::<B>();

        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_shutdown = shutdown.clone();

        let handle = thread::spawn(move || loop {
            if thread_shutdown.load(Ordering::SeqCst) {
                return;
            }
            let (old_a, old_b) = match ab_rx.recv() {
                Ok(v) => v,
                Err(_) => return,
            };
            let updated_b = updater.update(&old_a, &old_b);
            if let Err(_) = b_tx.send(updated_b) {
                return;
            };
        });

        Self {
            h: handle,
            tx: ab_tx,
            rx: b_rx,
            shutdown,
        }
    }

    pub fn send(&self, old_a: A, old_b: B) {
        self.tx.send((old_a, old_b)).unwrap();
    }

    pub fn receive(&self) -> B {
        self.rx.recv().expect("worker channel disconnected")
    }

    pub fn maybe_receive(&self) -> Option<B> {
        match self.rx.try_recv() {
            Ok(res) => Some(res),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(e) => panic!("worker channel error: {}", e),
        }
    }

    pub fn terminate(self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.h.join().expect("failed to join worker thread");
    }

    pub fn controller<W, T, LA, LB>(
        self,
        widget: W,
        lens_a: LA,
        lens_b: LB,
    ) -> AsyncUpdateController<W, T, A, B, LA, LB>
    where
        W: Widget<B>,
        T: Data,
        LA: Lens<T, A>,
        LB: Lens<T, B>,
    {
        AsyncUpdateController::new(self, widget, lens_a, lens_b)
    }
}

pub struct AsyncUpdateController<W, T, A, B, LA, LB>
where
    T: Data,
    A: Data,
    B: Data,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
    W: Widget<B>,
{
    waiting_on_updates: usize,
    widget: W,
    updater: AsyncUpdateWrapper<A, B>,
    lens_a: LA,
    lens_b: LB,
    t: PhantomData<T>,
}

impl<W, T, A, B, LA, LB> AsyncUpdateController<W, T, A, B, LA, LB>
where
    W: Widget<B>,
    T: Data,
    A: Data,
    B: Data,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
{
    pub fn new(updater: AsyncUpdateWrapper<A, B>, widget: W, lens_a: LA, lens_b: LB) -> Self {
        Self {
            waiting_on_updates: 0,
            updater,
            widget,
            lens_a,
            lens_b,
            t: PhantomData,
        }
    }
}

impl<W, T, A, B, LA, LB> Widget<T> for AsyncUpdateController<W, T, A, B, LA, LB>
where
    W: Widget<B>,
    T: Data + std::fmt::Debug,
    A: Data + Send + 'static,
    B: Data + Send + 'static + std::fmt::Debug,
    LA: Lens<T, A>,
    LB: Lens<T, B>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(updated_data_b) = self.updater.maybe_receive() {
            self.lens_b
                .with_mut(data, |data_b| *data_b = updated_data_b);
            ctx.request_paint();
            self.waiting_on_updates -= 1;
        } else {
            if self.waiting_on_updates > 0 {
                ctx.request_timer(std::time::Duration::from_millis(100));
            }
        }
        self.lens_b
            .with_mut(data, |data_b| self.widget.event(ctx, event, data_b, env))
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.lens_b.with(data, |data_b| {
            self.widget.lifecycle(ctx, event, data_b, env)
        })
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, new_data: &T, env: &Env) {
        let old_data_a: A = self.lens_a.with(old_data, |old_data_a| old_data_a.clone());
        let new_data_a: A = self.lens_a.with(new_data, |new_data_a| new_data_a.clone());

        if !old_data_a.same(&new_data_a) {
            self.lens_b.with(new_data, |new_data_b| {
                self.updater.send(new_data_a, new_data_b.clone());
            });
            self.waiting_on_updates += 1;
            ctx.request_timer(std::time::Duration::from_millis(100));
        }
        self.lens_b.with(old_data, |old_data_b| {
            self.lens_b.with(new_data, |new_data_b| {
                self.widget.update(ctx, old_data_b, new_data_b, env)
            })
        });
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        self.lens_b.with(data, |data_b| {
            self.widget.layout(layout_ctx, bc, data_b, env)
        })
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.lens_b
            .with(data, |data_b| self.widget.paint(ctx, data_b, env))
    }
}
