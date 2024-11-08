#![doc = include_str!("../README.md")]

use std::cmp::max;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem;
use std::ops::Add;
use std::ptr;
use std::rc::Rc;
use std::time::Duration;

use precision::*;

/// Options.
#[derive(Clone, Debug)]
pub struct Options {
    /// Number of iterations to perform.
    pub iterations: u64,
    /// Number of warm-up iterations to perform.
    pub warmup_iterations: u64,
    /// Minimum number of samples to collect.
    pub min_samples: usize,
    /// Maximum number of samples to collect.
    pub max_samples: usize,
    /// Maximum RSD to tolerate (in 0...100).
    pub max_rsd: f64,
    /// Maximum benchmark duration time.
    pub max_duration: Option<Duration>,
    /// Verbose output
    pub verbose: bool,
}

impl Default for Options {
    fn default() -> Self {
        let mut verbose = false;
        std::env::var("BENCHMARK_VERBOSE")
            .map(|_| verbose = true)
            .ok();

        Self {
            iterations: 1,
            warmup_iterations: 0,
            min_samples: 3,
            max_samples: 5,
            max_rsd: 5.0,
            verbose,
            max_duration: None,
        }
    }
}

/// A benchmark result.
#[derive(Clone)]
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
            unit: Unit::None,
        }
    }

    /// Compute the throughput in bits for a given volume of data.
    /// The volume is the amount of bytes processed in a single iteration.
    pub fn throughput_bits(self, mut volume: u128) -> Throughput {
        volume *= self.options.iterations as u128;
        volume *= 8;
        Throughput {
            volume: volume as f64,
            result: self,
            unit: Unit::Bits,
        }
    }

    /// Compute the throughput in bytes for a given volume of data.
    /// The volume is the amount of bytes processed in a single iteration.
    pub fn throughput_bytes(self, mut volume: u128) -> Throughput {
        volume *= self.options.iterations as u128;
        volume *= 8;
        Throughput {
            volume: volume as f64,
            result: self,
            unit: Unit::Bytes,
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

/// Unit
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[derive(Default)]
pub enum Unit {
    /// None
    #[default]
    None,
    /// Bytes
    Bytes,
    /// Bits
    Bits,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Unit::None => write!(f, ""),
            Unit::Bytes => write!(f, "B"),
            Unit::Bits => write!(f, "b"),
        }
    }
}


/// The result of a benchmark, as a throughput.
#[derive(Clone)]
pub struct Throughput {
    volume: f64,
    result: BenchResult,
    unit: Unit,
}

impl Throughput {
    /// The throughput as a floating point number.
    pub fn as_f64(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (max(1, self.result.as_ns()) as f64)
    }

    /// The throughput as an integer.
    pub fn as_u128(&self) -> u128 {
        self.volume as u128 * 1_000_000_000 / (max(1, self.result.as_ns()) as u128)
    }

    /// The throughput in kibibytes.
    pub fn as_kib(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / 1024.0
    }

    /// The throughput in mebibytes.
    pub fn as_mib(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / (1024.0 * 1024.0)
    }

    /// The throughput in gibibytes.
    pub fn as_gib(&self) -> f64 {
        self.volume * 1_000_000_000f64
            / (max(1, self.result.as_ns()) as f64)
            / (1024.0 * 1024.0 * 1024.0)
    }

    /// The throughput in kilobytes.
    pub fn as_kb(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / 1000.0
    }

    /// The throughput in megabytes.
    pub fn as_mb(&self) -> f64 {
        self.volume * 1_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / (1000.0 * 1000.0)
    }

    /// The throughput in gigabytes.
    pub fn as_gb(&self) -> f64 {
        self.volume * 1_000_000_000f64
            / (max(1, self.result.as_ns()) as f64)
            / (1000.0 * 1000.0 * 1000.0)
    }

    /// The throughput in kilobits.
    pub fn as_kb8(&self) -> f64 {
        self.volume * 8_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / 1000.0
    }

    /// The throughput in megabits.
    pub fn as_mb8(&self) -> f64 {
        self.volume * 8_000_000_000f64 / (max(1, self.result.as_ns()) as f64) / (1000.0 * 1000.0)
    }

    /// The throughput in gigabits.
    pub fn as_gb8(&self) -> f64 {
        self.volume * 8_000_000_000f64
            / (max(1, self.result.as_ns()) as f64)
            / (1000.0 * 1000.0 * 1000.0)
    }
}

impl Display for Throughput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.unit {
            Unit::None => match self.as_u128() {
                0..=999 => write!(f, "{:.2} /s", self.as_f64()),
                1_000..=999_999 => write!(f, "{:.2} K/s", self.as_kb()),
                1_000_000..=999_999_999 => write!(f, "{:.2} M/s", self.as_mb()),
                _ => write!(f, "{:.2} G/s", self.as_gb()),
            },
            Unit::Bytes => match self.as_u128() {
                0..=999 => write!(f, "{:.2} B/s", self.as_f64()),
                1_000..=999_999 => write!(f, "{:.2} KB/s", self.as_kb()),
                1_000_000..=999_999_999 => write!(f, "{:.2} MB/s", self.as_mb()),
                _ => write!(f, "{:.2} GB/s", self.as_gb()),
            },
            Unit::Bits => match self.as_u128() {
                0..=999 => write!(f, "{:.2} b/s", self.as_f64()),
                1_000..=999_999 => write!(f, "{:.2} Kb/s", self.as_kb()),
                1_000_000..=999_999_999 => write!(f, "{:.2} Mb/s", self.as_mb()),
                _ => write!(f, "{:.2} Gb/s", self.as_gb()),
            },
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

    fn run_once<F, G>(&self, options: Rc<Options>, f: &mut F) -> BenchResult
    where
        F: FnMut() -> G,
    {
        let iterations = options.iterations;
        let start = self.precision.now();
        for _ in 0..iterations {
            black_box(f());
        }
        let elapsed = self.precision.now() - start;
        BenchResult {
            elapsed,
            precision: self.precision.clone(),
            options,
        }
    }

    /// Run a single test.
    pub fn run<F, G>(&self, options: &Options, mut f: F) -> BenchResult
    where
        F: FnMut() -> G,
    {
        let options = Rc::new(options.clone());
        let max_samples = std::cmp::max(1, options.max_samples);
        let verbose = options.verbose;

        if verbose {
            println!("Starting a new benchmark.");
            if options.warmup_iterations > 0 {
                println!("Warming up for {} iterations.", options.warmup_iterations);
            }
        }
        for _ in 0..options.warmup_iterations {
            black_box(f());
        }
        let mut results = Vec::with_capacity(max_samples);
        let start = self.precision.now();
        for i in 1..=max_samples {
            if verbose {
                println!("Running iteration {}.", i);
            }
            let result = self.run_once(options.clone(), &mut f);
            results.push(result);
            if results.len() <= 1 {
                if verbose {
                    println!("Iteration {}: {}", i, results.last().unwrap());
                }
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
            if verbose {
                println!("Iteration {}: {:.2}s ± {:.2}%", i, mean, rsd);
            }
            if i >= options.min_samples && rsd < options.max_rsd {
                if verbose {
                    println!("Enough samples have been collected.");
                }
                break;
            }
            if let Some(max_duration) = options.max_duration {
                let elapsed =
                    Duration::from_secs((self.precision.now() - start).as_secs(&self.precision));
                if elapsed >= max_duration {
                    if verbose {
                        println!("Timeout.");
                    }
                    break;
                }
            }
        }
        let result = results.into_iter().min_by_key(|r| r.as_ns()).unwrap();
        if verbose {
            println!("Result: {}", result);
        }
        result
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
