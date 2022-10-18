use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Size, UpdateCtx, Widget,
};

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

    fn update(&mut self, _ctx: &mut UpdateCtx, old_data: &T, new_data: &T, _env: &Env) {
        let old_data_a: A = self.lens_a.with(old_data, |old_data_a| old_data_a.clone());
        let new_data_a: A = self.lens_a.with(new_data, |new_data_a| new_data_a.clone());
        if !old_data_a.same(&new_data_a) {
            self.updated = true
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
            let (old_a, old_b) = ab_rx.recv().unwrap();
            let updated_b = updater.update(&old_a, &old_b);
            b_tx.send(updated_b).unwrap();
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
    fn event(&mut self, ctx: &mut EventCtx, _event: &Event, data: &mut T, _env: &Env) {
        if let Some(updated_data_b) = self.updater.maybe_receive() {
            self.lens_b
                .with_mut(data, |data_b| *data_b = updated_data_b);
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, old_data: &T, new_data: &T, _env: &Env) {
        let old_data_a: A = self.lens_a.with(old_data, |old_data_a| old_data_a.clone());
        let new_data_a: A = self.lens_a.with(new_data, |new_data_a| new_data_a.clone());

        if !old_data_a.same(&new_data_a) {
            // TODO: set timer to try receiving
            self.lens_b.with(new_data, |new_data_b| {
                self.updater.send(new_data_a, new_data_b.clone());
            });
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
