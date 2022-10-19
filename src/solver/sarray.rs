use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::sync::Arc;

use crate::threads::RangeSplitter;

#[derive(Debug)]
pub struct SArray<T, const N: usize>
where
    T: Copy + Debug,
{
    parent: Option<Arc<SArray<T, N>>>,
    data: Arc<[Cell<T>; N]>,
    start: usize,
    end: usize,
}

unsafe impl<T, const N: usize> Send for SArray<T, N> where T: Copy + Debug {}

impl<T, const N: usize> SArray<T, N>
where
    T: Copy + Debug,
{
    pub fn full(v: T) -> Self {
        let data: [Cell<T>; N] = [v; N].map(|v| Cell::new(v));
        Self {
            parent: None,
            data: Arc::new(data),
            start: 0,
            end: N,
        }
    }

    fn cell(&self, n: usize) -> &Cell<T> {
        let n = n + self.start;
        assert!(n <= self.end, "out of bounds");
        &self.data[n]
    }

    pub fn get(&self, n: usize) -> T {
        self.cell(n).get()
    }

    pub fn set(&mut self, n: usize, value: T) {
        self.cell(n).set(value);
    }
}

impl<T, const N: usize> Default for SArray<T, N>
where
    T: Copy + Default + Debug,
{
    fn default() -> Self {
        Self::full(T::default())
    }
}

#[derive(Debug)]
pub struct SRefArray<T, const N: usize>
where
    T: Debug,
{
    parent: Option<Arc<SRefArray<T, N>>>,
    data: Arc<[RefCell<T>; N]>,
    start: usize,
    end: usize,
}

unsafe impl<T, const N: usize> Send for SRefArray<T, N> where T: Debug {}

impl<T, const N: usize> SRefArray<T, N>
where
    T: Debug,
{
    pub fn full(v: T) -> Self
    where
        T: Clone,
    {
        let data: [RefCell<T>; N] = [(); N].map(|_| RefCell::new(v.clone()));
        Self {
            parent: None,
            data: Arc::new(data),
            start: 0,
            end: N,
        }
    }

    fn cell(&self, n: usize) -> &RefCell<T> {
        let n = n + self.start;
        assert!(n <= self.end, "out of bounds");
        &self.data[n]
    }

    pub fn borrow(&self, n: usize) -> Ref<T> {
        self.cell(n).borrow()
    }

    pub fn borrow_mut(&mut self, n: usize) -> RefMut<T> {
        self.cell(n).borrow_mut()
    }

    pub fn get(&self, n: usize) -> T
    where
        T: Clone,
    {
        self.borrow(n).clone()
    }

    pub fn set(&mut self, n: usize, v: T) {
        *self.cell(n).borrow_mut() = v;
    }

    pub fn take(&mut self, n: usize) -> T
    where
        T: Default,
    {
        self.cell(n).take()
    }

    pub fn swap(&mut self, n1: usize, n2: usize) {
        self.cell(n1).swap(self.cell(n2))
    }

    pub fn replace(&mut self, n: usize, v: T) -> T {
        self.cell(n).replace(v)
    }
}

impl<T, const N: usize> Default for SRefArray<T, N>
where
    T: Copy + Default + std::fmt::Debug,
{
    fn default() -> Self {
        Self::full(T::default())
    }
}

trait SArrayLike<T>
where
    T: Clone,
{
    fn new(parent: Option<Arc<Self>>, data: T, start: usize, end: usize) -> Self;
    fn into_parent(self) -> Option<Arc<Self>>;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn data(&self) -> &T;
}

impl<T, const N: usize> SArrayLike<Arc<[Cell<T>; N]>> for SArray<T, N>
where
    T: Copy + Debug,
{
    fn new(parent: Option<Arc<Self>>, data: Arc<[Cell<T>; N]>, start: usize, end: usize) -> Self {
        Self {
            parent,
            data,
            start,
            end,
        }
    }
    fn into_parent(self) -> Option<Arc<Self>> {
        self.parent
    }
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
    fn data(&self) -> &Arc<[Cell<T>; N]> {
        &self.data
    }
}

impl<T, const N: usize> SArrayLike<Arc<[RefCell<T>; N]>> for SRefArray<T, N>
where
    T: Debug,
{
    fn new(
        parent: Option<Arc<Self>>,
        data: Arc<[RefCell<T>; N]>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            parent,
            data,
            start,
            end,
        }
    }
    fn into_parent(self) -> Option<Arc<Self>> {
        self.parent
    }
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
    fn data(&self) -> &Arc<[RefCell<T>; N]> {
        &self.data
    }
}

pub trait SArrayLen<T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<T, D> SArrayLen<D> for T
where
    T: SArrayLike<D> + Debug,
    D: Clone,
{
    fn len(&self) -> usize {
        self.end() - self.start()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait SArraySplit<T>
where
    Self: Sized,
{
    fn split(self, n: usize) -> Vec<Self>;
    fn join(v: Vec<Self>) -> Self;
}

impl<T, D> SArraySplit<D> for T
where
    T: SArrayLike<D> + Debug,
    D: Clone,
{
    fn split(self, n: usize) -> Vec<T> {
        let start = self.start();
        let end = self.end();
        let data = self.data().clone();
        let parent = Arc::new(self);
        let mut parts = vec![];
        for (m, n) in RangeSplitter::split(start, end, n) {
            parts.push(T::new(Some(parent.clone()), data.clone(), m, n));
        }
        parts
    }

    fn join(mut v: Vec<T>) -> T {
        let first = v.pop().unwrap();
        let parent = first.into_parent().take().unwrap();
        for other in v {
            let other_parent = other.into_parent().take().unwrap();
            assert!(Arc::ptr_eq(&parent, &other_parent));
            drop(other_parent);
        }
        Arc::try_unwrap(parent).expect("more children?")
    }
}

#[cfg(test)]
mod test {
    use super::{SArray, SArraySplit, SRefArray};
    use num::complex::Complex;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn test_sarray() {
        let sarray: SArray<i32, 12> = SArray::default();
        let mut v = sarray.split(3);
        let mut sc = v.pop().unwrap();
        let sb = v.pop().unwrap();
        let mut sa = v.pop().unwrap();
        let mut vb = sb.split(2);
        let mut sbb = vb.pop().unwrap();
        let mut sba = vb.pop().unwrap();

        let barrier = Arc::new(Barrier::new(4));

        let t_barrier = barrier.clone();
        let handle_sa = thread::spawn(move || {
            sa.set(0, 0);
            t_barrier.wait();
            sa.set(1, 1);
            sa.set(2, 2);
            sa.set(3, 3);
            sa
        });
        let t_barrier = barrier.clone();
        let handle_sba = thread::spawn(move || {
            t_barrier.wait();
            sba.set(0, 4);
            sba.set(1, 5);
            sba
        });
        let t_barrier = barrier.clone();
        let handle_sbb = thread::spawn(move || {
            t_barrier.wait();
            sbb.set(0, 6);
            sbb.set(1, 7);
            sbb
        });
        let t_barrier = barrier.clone();
        let handle_sc = thread::spawn(move || {
            sc.set(0, 8);
            t_barrier.wait();
            sc.set(1, 9);
            sc.set(2, 10);
            sc.set(3, 11);
            sc
        });

        let sa = handle_sa.join().unwrap();
        let sba = handle_sba.join().unwrap();
        let sbb = handle_sbb.join().unwrap();
        let sc = handle_sc.join().unwrap();

        let sb = SArray::join(vec![sba, sbb]);
        let sarray = SArray::join(vec![sa, sb, sc]);

        for i in 0..11 {
            assert_eq!(sarray.get(i), i as i32);
        }
    }

    #[test]
    fn test_srefarray() {
        let sarray: SRefArray<i32, 12> = SRefArray::default();
        let mut v = sarray.split(3);
        let mut sc = v.pop().unwrap();
        let sb = v.pop().unwrap();
        let mut sa = v.pop().unwrap();
        let mut vb = sb.split(2);
        let mut sbb = vb.pop().unwrap();
        let mut sba = vb.pop().unwrap();

        let barrier = Arc::new(Barrier::new(4));

        let t_barrier = barrier.clone();
        let handle_sa = thread::spawn(move || {
            sa.set(0, 0);
            t_barrier.wait();
            sa.set(1, 1);
            sa.set(2, 2);
            sa.set(3, 3);
            sa
        });
        let t_barrier = barrier.clone();
        let handle_sba = thread::spawn(move || {
            t_barrier.wait();
            sba.set(0, 4);
            sba.set(1, 5);
            sba
        });
        let t_barrier = barrier.clone();
        let handle_sbb = thread::spawn(move || {
            t_barrier.wait();
            sbb.set(0, 6);
            sbb.set(1, 7);
            sbb
        });
        let t_barrier = barrier.clone();
        let handle_sc = thread::spawn(move || {
            sc.set(0, 8);
            t_barrier.wait();
            sc.set(1, 9);
            sc.set(2, 10);
            sc.set(3, 11);
            sc
        });

        let sa = handle_sa.join().unwrap();
        let sba = handle_sba.join().unwrap();
        let sbb = handle_sbb.join().unwrap();
        let sc = handle_sc.join().unwrap();

        let sb = SRefArray::join(vec![sba, sbb]);
        let sarray = SRefArray::join(vec![sa, sb, sc]);

        for i in 0..11 {
            assert_eq!(sarray.get(i), i as i32);
        }
    }

    #[test]
    fn test_sarray_complex() {
        let carr: SArray<Complex<f64>, 4000> = SArray::default();
    }
}
