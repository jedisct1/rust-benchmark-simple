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
