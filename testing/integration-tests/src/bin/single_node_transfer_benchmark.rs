#[path = "benchmark_support/cli.rs"]
mod cli;
#[path = "benchmark_support/runner.rs"]
mod runner;

use cli::parse_bench_args;
use runner::{run_benchmark, BenchResult};

#[tokio::main(flavor = "current_thread")]
async fn main() -> BenchResult<()> {
    let args = parse_bench_args();
    run_benchmark(args).await
}
