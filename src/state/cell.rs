// use std::cell::Cell;
// use std::sync::atomic::AtomicBool;
// use std::sync::{Arc, Mutex};

// use crate::complex::{ci, cr, C};
// use crate::coord::Viewport;
// use crate::state::MbState;
// use crate::threads::{Solver, Split};

// pub const MB_CELL_STATE_HEIGHT: usize = 1500;
// pub const MB_CELL_STATE_WIDTH: usize = 1000;
// const ARRAY_L: usize = MB_CELL_STATE_HEIGHT * MB_CELL_STATE_WIDTH;

// type CArray = [Cell<C<f64>>; ARRAY_L];
// type IArray = [Cell<i16>; ARRAY_L];

// pub struct MbCellState {
//     iteration: i16,
//     c: CArray,
//     z: CArray,
//     i: IArray,
// }

// #[derive(Clone)]
// pub struct MbCellStateRef(Arc<MbCellStateShared>);

// impl MbCellStateRef {
//     pub fn slice(&self, row_start: usize, row_end: usize) -> Self {
//         Self(Arc::new(MbCellStateShared {
//             lock: Arc::new(Mutex::new(())),
//             row_start,
//             row_end,
//             parent: Some(self.clone()),
//             state: self.0.state.clone(),
//         }))
//     }
// }

// #[derive(Clone)]
// pub struct MbCellStateShared {
//     parent: Option<MbCellStateRef>,
//     lock: Arc<AtomicBool>,
//     state: Arc<MbCellState>,
//     row_start: usize,
//     row_end: usize,
// }

// impl Split for MbCellStateRef {
//     fn split_parts(self, n: usize) -> Vec<Self> {
//         assert!(n > 0);
//         self.0.lock.lock().unwrap();

//         let len = self.0.row_end - self.0.row_start;
//         let size = len / n;
//         let size_xtra = len % n;

//         let mut start = self.0.row_start;
//         let mut end = start + size;
//         let mut parts: Vec<Self> = vec![];
//         for i in 0..n {
//             if i < size_xtra {
//                 end += 1
//             }
//             parts.push(self.slice(start, end));
//             start = end;
//             end += size;
//         }
//         parts
//     }
//     fn join_parts(parts: Vec<Self>) -> Self {
//         let parent = parts[0]
//             .0
//             .parent
//             .as_ref()
//             .expect("cannot join non-split!")
//             .clone();

//         parent
//     }
// }

// pub struct MbCellSolver {
//     treshold: f64,
// }

// impl Default for MbCellSolver {
//     fn default() -> Self {
//         Self { treshold: 2.0 }
//     }
// }

// impl MbState for MbCellStateShared {
//     fn initialize(width: usize, height: usize, grid: &Viewport) -> Self {
//         assert!(width == MB_CELL_STATE_WIDTH, "wrong width");
//         assert!(height == MB_CELL_STATE_HEIGHT, "wrong height");
//         let x_b = cr(grid.x.min);
//         let x_m = cr(grid.x.length() / (width as f64 - 1.0));
//         let y_b = cr(grid.y.min);
//         let y_m = cr(grid.y.length() / (height as f64 - 1.0));

//         let c: CArray = [(); ARRAY_L].map(|_| Cell::new(cr(0.0)));
//         let z: CArray = [(); ARRAY_L].map(|_| Cell::new(cr(0.0)));
//         let i: IArray = [(); ARRAY_L].map(|_| Cell::new(-1));

//         for y in 0..MB_CELL_STATE_HEIGHT {
//             for x in 0..MB_CELL_STATE_WIDTH {
//                 let cx = x_b + x_m * x as f64;
//                 let cy = y_b + y_m * y as f64;
//                 let cc = cx + cy * ci(1.0);
//                 c[x * y].set(cc);
//                 z[x * y].set(cc);
//             }
//         }

//         Self {
//             parent: None,
//             lock: Arc::new(Mutex::new(())),
//             row_start: 0,
//             row_end: MB_CELL_STATE_HEIGHT,
//             state: Arc::new(MbCellState {
//                 iteration: 0,
//                 c,
//                 z,
//                 i,
//             }),
//         }
//     }
// }

// impl Solver<MbCellState> for MbCellSolver {
//     fn solve(&self, _: MbCellState) -> MbCellState {
//         todo!()
//     }
// }
