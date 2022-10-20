#![allow(clippy::new_without_default)]
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use image::RgbImage;
use painter::Rainbow;

use crate::coord::{Point, Viewbox};
use crate::painter::{ColorScale, IValuePainter, Painter};
use crate::solver::{MbState, MbVecSolver, MbVecState, Solver};
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
        let initial: T = position.into();
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
        self.state = self.solver.solve(self.position.into());
    }

    pub fn set_position(&mut self, position: Viewbox) {
        self.position = position;
        self.state = self.solver.solve(self.position.into());
    }

    pub fn zoom(&mut self, factor: f64) {
        self.position.zoom(factor);
        self.state = self.solver.solve(self.position.into());
    }

    pub fn pan(&mut self, x: i64, y: i64) {
        self.position.center = self.position.center.add(&Point::new(x, y));
        self.state = self.solver.solve(self.position.into());
    }

    pub fn pan_relative(&mut self, x: f64, y: f64) {
        let nx = f64::round(x * self.position.width as f64) as i64;
        let ny = f64::round(y * self.position.height as f64) as i64;
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

#[derive(Copy, Clone, Debug)]
pub enum MAction {
    Resize(i64, i64),
    Pan(i64, i64),
    PanRelative(f64, f64),
    Zoom(f64),
    Reset(i64, i64),
}

pub struct MandelbrotWorker {
    tx: Sender<MAction>,
    images: Arc<RwLock<VecDeque<RgbImage>>>,
    shutdown: Arc<AtomicBool>,
    last_event_t: Instant,
    cooldown: Duration,
}

impl MandelbrotWorker {
    fn spawn_receive_images(
        rx: Receiver<RgbImage>,
        images: Arc<RwLock<VecDeque<RgbImage>>>,
        shutdown: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || loop {
            if shutdown.load(Ordering::SeqCst) {
                return;
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(img) => images.write().unwrap().push_back(img),
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
            let mut m: Option<Mandelbrot<MbVecState>> = None;
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    return;
                }
                let repaint = match rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(MAction::Reset(w, h)) => {
                        m = Some(Mandelbrot::initialize::<MbVecSolver>(w, h));
                        true
                    }
                    Ok(MAction::Resize(w, h)) => {
                        let m =
                            m.get_or_insert_with(|| Mandelbrot::initialize::<MbVecSolver>(w, h));
                        m.resize(w, h);
                        true
                    }
                    Ok(MAction::Pan(x, y)) => match m {
                        Some(ref mut m) => {
                            m.pan(x, y);
                            true
                        }
                        None => false,
                    },
                    Ok(MAction::PanRelative(x, y)) => match m {
                        Some(ref mut m) => {
                            m.pan_relative(x, y);
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
                        if let Err(_) = tx.send(m.paint(Rainbow, 100)) {
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
        let images = Arc::new(RwLock::<VecDeque<RgbImage>>::new(VecDeque::new()));
        let shutdown = Arc::new(AtomicBool::new(false));

        Self::spawn_receive_images(rx_img, images.clone(), shutdown.clone());
        Self::spawn_mandelbrot(rx_actions, tx_img, shutdown.clone());

        Self {
            tx: tx_actions,
            images,
            shutdown,
            last_event_t: Instant::now(),
            cooldown: Duration::from_millis(50),
        }
    }

    fn send(&self, action: MAction) {
        self.tx.send(action).unwrap();
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
        self.images.read().unwrap().len()
    }

    pub fn next_image(&self) -> Option<RgbImage> {
        self.images.write().unwrap().pop_front()
    }
}

impl Drop for MandelbrotWorker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}
