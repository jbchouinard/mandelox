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

pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
}

impl Unit {
    pub fn format(&self, d: &Duration, width: usize) -> String {
        let (symbol, value) = match self {
            Self::Nanosecond => ("ns", d.as_nanos()),
            Self::Microsecond => ("µs", d.as_micros()),
            Self::Millisecond => ("ms", d.as_millis()),
            Self::Second => ("s", d.as_secs() as u128),
        };
        format!("{:>width$}{:<2}", value, symbol)
    }

    pub fn scaled(d: &Duration, treshold: u128) -> Self {
        if d.as_nanos() < treshold {
            Self::Nanosecond
        } else if d.as_micros() < treshold {
            Self::Microsecond
        } else if d.as_millis() < treshold {
            Self::Millisecond
        } else {
            Self::Second
        }
    }
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

    fn run(&self) -> Duration {
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
        println!();
        stdout().flush().unwrap();
    }
    pub fn show(&self) {
        println!(
            "  {: <30} {: >8}   {: >8}",
            "benchmark", "total", "per_call"
        );
        for (name, iterations, t) in &self.results {
            let t_per_call = t.div_f64(*iterations as f64);
            println!(
                "  {: <30} {}   {}",
                name,
                Unit::scaled(t, 100000).format(t, 6),
                Unit::scaled(&t_per_call, 100000).format(&t_per_call, 6),
            )
        }
        stdout().flush().unwrap();
    }

    pub fn write_csv(&self, filename: &str) {
        let mut lines: Vec<String> = vec!["benchmark,total_us,iterations,per_call_us".to_string()];
        for (name, iterations, t) in &self.results {
            lines.push(format!(
                "{},{},{},{}",
                name,
                t.as_micros(),
                iterations,
                t.as_micros() / *iterations as u128,
            ));
        }
        lines.push("".to_string());
        fs::write(filename, lines.join("\n")).unwrap();
    }

    pub fn report(&mut self, name: &str) {
        print!("Benchmark: {}", name);
        self.run();
        self.show();
        self.write_csv(&format!("benchmark_{}.csv", name))
    }
}
