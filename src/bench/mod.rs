use std::fs;
use std::io::{stdout, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Benchmark {
    f: Rc<dyn Fn()>,
    name: String,
    iterations: usize,
}

impl Benchmark {
    pub fn iter<F: Fn() + 'static>(name: &str, n: usize, f: F) -> Self {
        Self {
            f: Rc::new(f),
            name: name.to_string(),
            iterations: n,
        }
    }

    pub fn once<F: Fn() + 'static>(name: &str, f: F) -> Self {
        Self::iter(name, 1, f)
    }

    pub fn run(&self) -> Duration {
        let start = Instant::now();
        for _ in 0..self.iterations {
            (self.f)();
        }
        Instant::now() - start
    }
}

pub struct BenchmarkReport {
    benches: Vec<Benchmark>,
    results: Vec<(String, usize, Duration)>,
}

impl BenchmarkReport {
    pub fn new() -> Self {
        Self {
            benches: vec![],
            results: vec![],
        }
    }

    pub fn add_bench(&mut self, bench: Benchmark) {
        self.benches.push(bench);
    }

    pub fn add_benches(&mut self, benches: &[Benchmark]) {
        for bench in benches {
            self.benches.push(bench.clone())
        }
    }

    pub fn with_benches(benches: &[Benchmark]) -> Self {
        let mut this = Self::new();
        this.add_benches(benches);
        this
    }

    pub fn run(&mut self) {
        for bench in &self.benches {
            let t = bench.run();
            self.results
                .push((bench.name.to_string(), bench.iterations, t));
            print!(".");
            stdout().flush().unwrap();
        }
        print!("\n\n");
        stdout().flush().unwrap();
    }

    pub fn show(&self) {
        for (name, iterations, t) in &self.results {
            println!(
                "{}\n  per call: {}μs\n  total: {}ms\n",
                name,
                t.as_micros() / *iterations as u128,
                t.as_millis()
            )
        }
        stdout().flush().unwrap();
    }

    pub fn write_csv(&self, filename: &str) {
        let mut lines: Vec<String> = vec!["benchmark,per_call_us,iterations,total_ms".to_string()];
        for (name, iterations, t) in &self.results {
            lines.push(format!(
                "{},{},{},{}",
                name,
                t.as_micros(),
                iterations,
                t.as_millis()
            ));
        }
        lines.push("".to_string());
        fs::write(filename, lines.join("\n")).unwrap();
    }
}
