use std::cell::Cell;
use std::fmt::Debug;
use std::sync::Arc;

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

trait SArrayLike<D>
where
    D: Clone,
{
    fn new(parent: Option<Arc<Self>>, data: D, start: usize, end: usize) -> Self;
    fn into_parent(self) -> Option<Arc<Self>>;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn data(&self) -> &D;
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

pub trait SArraySplit<D>
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
        let len = self.end() - self.start();
        let size = len / n;
        let size_xtra = len % n;
        let mut start = self.start();
        let mut end = self.start() + size;

        let data = self.data().clone();
        let parent = Arc::new(self);
        let mut parts = vec![];

        for i in 0..n {
            if i < size_xtra {
                end += 1;
            }
            parts.push(T::new(Some(parent.clone()), data.clone(), start, end));
            start = end;
            end += size;
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

impl<T, const N: usize> SArray<T, N>
where
    T: Copy + std::fmt::Debug,
{
    pub fn new(v: T) -> Self {
        let data: [Cell<T>; N] = [v; N].map(|v| Cell::new(v));
        Self {
            parent: None,
            data: Arc::new(data),
            start: 0,
            end: N,
        }
    }

    pub fn get(&self, n: usize) -> T {
        let n = n + self.start;
        assert!(n <= self.end, "out of bounds");
        self.data[n].get()
    }

    pub fn set(&mut self, n: usize, value: T) {
        let n = n + self.start;
        assert!(n <= self.end, "out of bounds");
        self.data[n].set(value);
    }
}

impl<T, const N: usize> Default for SArray<T, N>
where
    T: Copy + Default + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[cfg(test)]
mod test {
    use super::{SArray, SArraySplit};
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
}
