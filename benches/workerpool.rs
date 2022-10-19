use mandelox::bench::{Benchmark, BenchmarkReport};
use mandelox::threads::{vectorize, Call, Threaded};

const TASK_MEM_SIZE: usize = 10000;
const TASK_CPU_SIZE: usize = 10000;
const ITN: usize = 10;

fn square(mut x: i64) -> i64 {
    for _ in 0..TASK_CPU_SIZE {
        x = (x * x) + x;
    }
    x
}

fn vrange(n: usize) -> Vec<i64> {
    (0..n).map(|n| n as i64).collect()
}

fn bench_threadpool(threads: usize) -> Benchmark {
    let name = &format!("square-t{}-n{}", threads, TASK_MEM_SIZE);

    if threads == 0 {
        let f = vectorize(square);
        let c = move || {
            f.call(vrange(TASK_MEM_SIZE));
        };
        Benchmark::iter(name, ITN, c)
    } else {
        let f = vectorize(square).threadpool(threads);
        let c = move || {
            f.call(vrange(TASK_MEM_SIZE));
        };
        Benchmark::iter(name, ITN, c)
    }
}

fn main() {
    BenchmarkReport::with_benches(&[
        bench_threadpool(0),
        bench_threadpool(1),
        bench_threadpool(2),
        bench_threadpool(4),
        bench_threadpool(8),
    ])
    .report("workerpool");
}
