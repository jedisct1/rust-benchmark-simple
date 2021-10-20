//! # benchmark-simple
//!
//! A tiny benchmarking library for Rust.
//!
//! - Trivial to use
//! - Works pretty much everywhere, including WebAssembly (WASI, but also in-browser)
//!
//! ```rust
//! use benchmark_simple::*;
//!
//! fn test_function() {
//!     // ...
//! }
//!
//! let bench = Bench::new();
//! let options = Options::default();
//! let res = bench.run(&options, || test_function());
//! println!("result: {}", res);
//! ```
//!
//! Throughput computation:
//!
//! ```rust
//! use benchmark_simple::*;
//!
//! fn test_function(m: &mut [u8]) {
//!     // ...
//! }
//!
//! let mut m = vec![0u8; 1_000_000];
//! let bench = Bench::new();
//! let options = Options::default();
//! let res = bench.run(&options, || test_function(&mut m));
//! let throughput = res.throughput(m.len() as _);
//! println!("throughput: {}", throughput);
//! ```

use precision::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem;
use std::ops::Add;
use std::ptr;
use std::rc::Rc;

/// Options.
#[derive(Clone, Debug)]
pub struct Options {
    /// Number of iterations to perform.
    pub iterations: u64,
    /// Maximum number of samples to collect.
    pub max_samples: usize,
    /// Maximum RSD to tolerate (in 0...100)
    pub max_rsd: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            iterations: 1,
            max_samples: 3,
            max_rsd: 5.0,
        }
    }
}

/// A benchmark result.
pub struct BenchResult {
    elapsed: Elapsed,
    precision: Precision,
    options: Rc<Options>,
}

impl Add for BenchResult {
    type Output = BenchResult;

    fn add(self, other: BenchResult) -> Self::Output {
        BenchResult {
            elapsed: self.elapsed + other.elapsed,
            precision: self.precision,
            options: self.options,
        }
    }
}

impl BenchResult {
    /// Returns the number of ticks.
    pub fn ticks(&self) -> u64 {
        self.elapsed.ticks()
    }

    /// Returns the elapsed time in seconds.
    pub fn as_secs(&self) -> u64 {
        self.elapsed.as_secs(&self.precision)
    }

    /// Returns the elapsed time in seconds (floating point).
    pub fn as_secs_f64(&self) -> f64 {
        self.elapsed.as_secs_f64(&self.precision)
    }

    /// Returns the elapsed time in milliseconds.
    pub fn as_millis(&self) -> u64 {
        self.elapsed.as_millis(&self.precision)
    }

    /// Returns the elapsed time in nanoseconds.
    pub fn as_ns(&self) -> u64 {
        self.elapsed.as_ns(&self.precision)
    }

    /// Compute the throughput for a given volume of data.
    /// The volume is the amount of bytes processed in a single iteration.
    pub fn throughput(self, mut volume: u128) -> Throughput {
        volume *= self.options.iterations as u128;
        Throughput {
            volume: volume as f64,
            result: self,
        }
    }
}

impl Display for BenchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}s", self.as_secs_f64())
    }
}

impl Debug for BenchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// The result of a benchmark, as a throughput.
pub struct Throughput {
    volume: f64,
    result: BenchResult,
}

impl Throughput {
    /// The throughput as a floating point number.
    pub fn as_f64(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64)
    }

    /// The throughput as an integer.
    pub fn as_u128(&self) -> u128 {
        self.volume as u128 * 1_000_000_000 / (self.result.as_ns() as u128)
    }

    /// The throughput in kibibytes.
    pub fn as_kib(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / 1024.0
    }

    /// The throughput in mebibytes.
    pub fn as_mib(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / (1024.0 * 1024.0)
    }

    /// The throughput in gibibytes.
    pub fn as_gib(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / (1024.0 * 1024.0 * 1024.0)
    }

    /// The throughput in kilobytes.
    pub fn as_kb(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / 1000.0
    }

    /// The throughput in megabytes.
    pub fn as_mb(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / (1000.0 * 1000.0)
    }

    /// The throughput in gigabytes.
    pub fn as_gb(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (self.result.as_ns() as f64) / (1000.0 * 1000.0 * 1000.0)
    }
}

impl Display for Throughput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.as_u128() {
            0..=999 => write!(f, "{:.2} /s", self.as_f64()),
            1_000..=999_999 => write!(f, "{:.2} K/s", self.as_kb()),
            1_000_000..=999_999_999 => write!(f, "{:.2} M/s", self.as_mb()),
            _ => write!(f, "{:.2} G/s", self.as_gb()),
        }
    }
}

impl Debug for Throughput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// A benchmarking environment.
#[derive(Clone)]
pub struct Bench {
    precision: Precision,
}

impl Bench {
    /// Create a new benchmarking environment.
    pub fn new() -> Self {
        let precision = Precision::new(Default::default()).unwrap();
        Bench { precision }
    }

    pub fn run_once<F>(&self, options: Rc<Options>, f: &mut F) -> BenchResult
    where
        F: FnMut(),
    {
        let iterations = options.iterations;
        let start = self.precision.now();
        for _ in 0..iterations {
            f();
        }
        let elapsed = self.precision.now() - start;
        BenchResult {
            elapsed,
            precision: self.precision.clone(),
            options,
        }
    }

    /// Run a single test.
    pub fn run<F>(&self, options: &Options, mut f: F) -> BenchResult
    where
        F: FnMut(),
    {
        let options = Rc::new(options.clone());
        let max_samples = std::cmp::max(1, options.max_samples);
        let mut results = Vec::with_capacity(max_samples as usize);
        for _ in 0..max_samples {
            let result = self.run_once(options.clone(), &mut f);
            results.push(result);
            if results.len() <= 1 {
                continue;
            }
            let mean = results.iter().map(|r| r.as_secs_f64()).sum::<f64>() / results.len() as f64;
            let std_dev = (results
                .iter()
                .map(|r| (r.as_secs_f64() - mean).powi(2))
                .sum::<f64>()
                / (results.len() - 1) as f64)
                .sqrt();
            let rsd = std_dev * 100.0 / mean;
            if rsd < options.max_rsd {
                break;
            }
        }
        results.into_iter().min_by_key(|r| r.as_ns()).unwrap()
    }
}

impl Default for Bench {
    fn default() -> Self {
        Self::new()
    }
}

/// Force the compiler to avoid optimizing away a value that is computed
/// for benchmarking purposes, but not used afterwards.
#[inline(never)]
pub fn black_box<T>(dummy: T) -> T {
    let ret = unsafe { ptr::read_volatile(&dummy) };
    mem::forget(dummy);
    ret
}
