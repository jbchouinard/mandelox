#![allow(clippy::new_without_default)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use image::RgbImage;

use crate::coord::{Point, Viewbox};
use crate::painter::{ColorScale, IValuePainter, Painter, Rainbow};
use crate::solver::{D2ArrayLike, MbState, Solver};
use crate::threads::{Join, Split};

pub mod bench;
mod complex;
pub mod coord;
#[cfg(feature = "gui")]
pub mod gui;
pub mod painter;
pub mod solver;
pub mod threads;

pub struct Mandelbrot<T> {
    pub solver: Box<dyn Solver<T>>,
    pub position: Viewbox,
    pub state: T,
}

impl<T> Mandelbrot<T>
where
    T: MbState + Split + Join + Send + 'static,
{
    pub fn initialize<S>(width: i64, height: i64) -> Self
    where
        S: Solver<T> + Default + Clone + Send + 'static,
    {
        let position = Viewbox::initial(width, height);
        let solver = S::default().threaded(num_cpus::get_physical());
        let initial: T = position.generate_complex_coordinates().into();
        let solved = solver.solve(initial);
        Self {
            position,
            state: solved,
            solver: Box::new(solver),
        }
    }

    pub fn resize(&mut self, width: i64, height: i64) {
        self.position.height = height;
        self.position.width = width;
        self.state = self
            .solver
            .solve(self.position.generate_complex_coordinates().into());
    }

    pub fn set_position(&mut self, position: Viewbox) {
        self.position = position;
        self.state = self
            .solver
            .solve(self.position.generate_complex_coordinates().into());
    }

    pub fn zoom(&mut self, factor: f64) {
        self.position.zoom(factor);
        self.state = self
            .solver
            .solve(self.position.generate_complex_coordinates().into());
    }

    pub fn pan(&mut self, x: i64, y: i64) {
        self.position.center = self.position.center.add(&Point::new(x, y));
        self.state = self
            .solver
            .solve(self.position.generate_complex_coordinates().into());
    }

    pub fn pan_relative(&mut self, x: f64, y: f64) {
        let nx = (x * self.position.width as f64).round() as i64;
        let ny = (y * self.position.height as f64).round() as i64;
        self.pan(nx, ny);
    }

    pub fn paint<C>(&self, color: C, max_i_value: i16) -> RgbImage
    where
        C: ColorScale,
    {
        let painter = IValuePainter::new(color, max_i_value);
        painter.paint(&self.state)
    }
}

impl<T> Mandelbrot<T>
where
    T: D2ArrayLike + MbState + Split + Join + Send + 'static,
{
    pub fn pan_fast_vertical(&mut self, y: i64) {
        self.position.center = self.position.center.add(&Point::new(0, y));
        let new_coord_rows = self.position.generate_complex_coordinates().copy_rows(-y);
        let new_state_rows = self.solver.solve(new_coord_rows.into());
        self.state.shift_rows(-y, Some(&new_state_rows));
    }
    pub fn pan_fast_vertical_relative(&mut self, y: f64) {
        let ny = (y * self.position.height as f64).round() as i64;
        if ny != 0 {
            self.pan_fast_vertical(ny)
        }
    }

    pub fn pan_fast_horizontal(&mut self, x: i64) {
        self.position.center = self.position.center.add(&Point::new(x, 0));
        let new_coord_cols = self.position.generate_complex_coordinates().copy_cols(-x);
        let new_stat_cols = self.solver.solve(new_coord_cols.into());
        self.state.shift_cols(-x, Some(&new_stat_cols));
    }
    pub fn pan_fast_horizontal_relative(&mut self, x: f64) {
        let nx = (x * self.position.width as f64).round() as i64;
        if nx != 0 {
            self.pan_fast_horizontal(nx)
        }
    }
}

// #[cfg(target_arch = "aarch64")]
pub mod defaults {
    use crate::solver::{VecSolver, VecState};
    pub type Solver = VecSolver;
    pub type State = VecState;
}

// #[cfg(target_arch = "x86_64")]
// pub mod defaults {
//     use crate::solver::{SimdVecSolver, SimdVecState};
//     pub type Solver = SimdVecSolver;
//     pub type State = SimdVecState;
// }

pub fn mandelbrot(width: i64, height: i64) -> Mandelbrot<defaults::State> {
    Mandelbrot::<defaults::State>::initialize::<defaults::Solver>(width, height)
}

#[derive(Copy, Clone, Debug)]
pub enum MAction {
    Resize(i64, i64),
    Pan(i64, i64),
    PanRelative(f64, f64),
    Zoom(f64),
    Reset(i64, i64),
}

pub trait ActionQueue {
    fn add(&self, action: MAction);
}

pub struct SyncActionQueue {
    tx: Sender<MAction>,
}

impl SyncActionQueue {
    pub fn new(tx: Sender<MAction>) -> Self {
        Self { tx }
    }
}

impl ActionQueue for SyncActionQueue {
    fn add(&self, action: MAction) {
        self.tx.send(action).unwrap()
    }
}

pub struct BatchActionQueue {
    q: Arc<RwLock<Vec<MAction>>>,
}

impl BatchActionQueue {
    fn spawn_q_sender(q: Arc<RwLock<Vec<MAction>>>, tx: Sender<MAction>) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(100));
            let mut resize_x: i64 = 0;
            let mut resize_y: i64 = 0;
            let mut pan_x: i64 = 0;
            let mut pan_y: i64 = 0;
            let mut pan_rel_x: f64 = 0.0;
            let mut pan_rel_y: f64 = 0.0;
            let mut zoom: f64 = 1.0;
            let messages: Vec<MAction> = std::mem::take(q.write().unwrap().as_mut());
            for message in messages {
                match message {
                    MAction::Resize(x, y) => {
                        resize_x = x;
                        resize_y = y;
                    }
                    MAction::Pan(x, y) => {
                        pan_x += x;
                        pan_y += y;
                    }
                    MAction::PanRelative(x, y) => {
                        pan_rel_x += x;
                        pan_rel_y += y;
                    }
                    MAction::Zoom(f) => {
                        zoom *= f;
                    }
                    MAction::Reset(_, _) => {
                        tx.send(message).unwrap();
                        continue;
                    }
                }
            }
            if resize_x != 0 && resize_y != 0 {
                tx.send(MAction::Resize(resize_x, resize_y)).unwrap();
            }
            if pan_x != 0 || pan_y != 0 {
                tx.send(MAction::Pan(pan_x, pan_y)).unwrap();
            }
            if pan_rel_x != 0.0 || pan_rel_y != 0.0 {
                tx.send(MAction::PanRelative(pan_rel_x, pan_rel_y)).unwrap();
            }
            if zoom != 1.0 {
                tx.send(MAction::Zoom(zoom)).unwrap();
            }
        })
    }

    pub fn new(tx: Sender<MAction>) -> Self {
        let q = Arc::new(RwLock::new(vec![]));
        BatchActionQueue::spawn_q_sender(q.clone(), tx);
        Self { q }
    }
}

impl ActionQueue for BatchActionQueue {
    fn add(&self, action: MAction) {
        self.q.write().unwrap().push(action);
    }
}

pub struct MandelbrotWorker {
    queue: Box<dyn ActionQueue>,
    images: Arc<RwLock<Option<RgbImage>>>,
    shutdown: Arc<AtomicBool>,
}

impl MandelbrotWorker {
    fn spawn_receive_images(
        rx: Receiver<RgbImage>,
        images: Arc<RwLock<Option<RgbImage>>>,
        shutdown: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            if shutdown.load(Ordering::SeqCst) {
                return;
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(img) => {
                    images.write().unwrap().replace(img);
                }
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => return,
            }
        })
    }

    fn spawn_mandelbrot(
        rx: Receiver<MAction>,
        tx: Sender<RgbImage>,
        shutdown: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut m: Option<Mandelbrot<defaults::State>> = None;
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    return;
                }
                let repaint = match rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(MAction::Reset(w, h)) => {
                        m = Some(mandelbrot(w, h));
                        true
                    }
                    Ok(MAction::Resize(w, h)) => {
                        let m = m.get_or_insert_with(|| mandelbrot(w, h));
                        m.resize(w, h);
                        true
                    }
                    Ok(MAction::Pan(x, y)) => match m {
                        Some(ref mut m) => {
                            if x != 0 {
                                m.pan_fast_vertical(y);
                            }
                            if y != 0 {
                                m.pan_fast_horizontal(x);
                            }
                            true
                        }
                        None => false,
                    },
                    Ok(MAction::PanRelative(x, y)) => match m {
                        Some(ref mut m) => {
                            if y != 0.0 {
                                m.pan_fast_vertical_relative(y);
                            }
                            if x != 0.0 {
                                m.pan_fast_horizontal_relative(x);
                            }
                            true
                        }
                        None => false,
                    },
                    Ok(MAction::Zoom(factor)) => match m {
                        Some(ref mut m) => {
                            m.zoom(factor);
                            true
                        }
                        None => false,
                    },
                    Err(RecvTimeoutError::Timeout) => false,
                    Err(RecvTimeoutError::Disconnected) => return,
                };
                if repaint {
                    if let Some(ref m) = m {
                        if tx.send(m.paint(Rainbow, 100)).is_err() {
                            return;
                        }
                    }
                }
            }
        })
    }

    pub fn new() -> Self {
        let (tx_actions, rx_actions) = channel::<MAction>();
        let (tx_img, rx_img) = channel::<RgbImage>();
        let images = Arc::new(RwLock::<Option<RgbImage>>::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));

        Self::spawn_receive_images(rx_img, images.clone(), shutdown.clone());
        Self::spawn_mandelbrot(rx_actions, tx_img, shutdown.clone());

        Self {
            queue: Box::new(BatchActionQueue::new(tx_actions)),
            images,
            shutdown,
        }
    }

    fn send(&self, action: MAction) {
        self.queue.add(action);
    }

    pub fn reset(&self, width: i64, height: i64) {
        self.send(MAction::Reset(width, height));
    }

    pub fn resize(&self, width: i64, height: i64) {
        self.send(MAction::Resize(width, height));
    }

    pub fn pan(&self, x: i64, y: i64) {
        self.send(MAction::Pan(x, y));
    }

    pub fn pan_relative(&self, x: f64, y: f64) {
        self.send(MAction::PanRelative(x, y))
    }

    pub fn zoom(&self, factor: f64) {
        self.send(MAction::Zoom(factor))
    }

    pub fn images_count(&self) -> usize {
        usize::from(self.images.read().unwrap().is_some())
    }

    pub fn next_image(&self) -> Option<RgbImage> {
        self.images.write().unwrap().take()
    }
}

impl Drop for MandelbrotWorker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}
