use precision::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem;
use std::ptr;

/// Options.
#[derive(Default, Clone, Debug)]
pub struct Options {
    /// Number of iterations to perform.
    iterations: Option<u64>,
}

/// A benchmark result.
pub struct BenchResult {
    elapsed: Elapsed,
    precision: Precision,
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
    pub fn throughput(self, volume: u128) -> Throughput {
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
            _ => write!(f, "{:.2}G/s", self.as_gb()),
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

    /// Run a single test.
    pub fn run<F>(&self, options: Option<&Options>, mut f: F) -> BenchResult
    where
        F: FnMut(),
    {
        let iterations = options.map(|o| o.iterations.unwrap_or(1)).unwrap_or(1);
        let start = self.precision.now();
        for _ in 0..iterations {
            f();
        }
        let elapsed = self.precision.now() - start;
        BenchResult {
            elapsed,
            precision: self.precision.clone(),
        }
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
