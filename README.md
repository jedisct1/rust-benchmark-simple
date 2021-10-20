# benchmark-simple

A tiny benchmarking library for Rust.

- Trivial to use
- Works pretty much everywhere, including WebAssembly (WASI, but also in-browser)

```rust
use benchmark_simple::*;

fn test_function() {
    // ...
}

let bench = Bench::new();
let options = Options::default();
let res = bench.run(&options, || test_function());
println!("result: {}", res);
```

Throughput computation:

```rust
use benchmark_simple::*;

fn test_function(m: &mut [u8]) {
    // ...
}

let mut m = vec![0u8; 1_000_000];
let bench = Bench::new();
let options = Options::default();
let res = bench.run(&options, || test_function(&mut m));
let throughput = res.throughput(m.len() as _);
println!("throughput: {}", throughput);
```

Options:

```rust
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
    pub max_duration: Option<std::time::Duration>,
    /// Verbose output
    pub verbose: bool,
}
```

Benchmark results can be made verbose by setting `verbose` to `true` in the
`Options` struct, or by defining a `BENCHMARK_VERBOSE` environment variable.