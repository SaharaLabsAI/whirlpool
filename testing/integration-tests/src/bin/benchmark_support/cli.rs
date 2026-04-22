use std::env;

const DEFAULT_DURATION_SECONDS: u64 = 120;
const DEFAULT_SENDER_ACCOUNTS: usize = 2_000;
const DEFAULT_RECIPIENT_ACCOUNTS: usize = 2_000;
const DEFAULT_BLOCK_INTERVAL_MS: u64 = 1_000;

pub struct BenchArgs {
    pub duration_seconds: u64,
    pub sender_accounts: usize,
    pub recipient_accounts: usize,
    pub block_interval_ms: u64,
}

pub fn parse_bench_args() -> BenchArgs {
    parse_bench_args_impl()
}

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [--duration-seconds <u64>] [--sender-accounts <usize>] [--recipient-accounts <usize>] [--block-interval-ms <u64>]\n\
         Env fallbacks: BENCHMARK_DURATION_SECONDS, BENCH_SENDER_ACCOUNTS, BENCH_RECIPIENT_ACCOUNTS, BENCH_BLOCK_INTERVAL_MS\n\
         Legacy arg accepted but ignored: --whirlpool-node-bin <path>"
    );
}

fn parse_u64_arg(program: &str, name: &str, raw: Option<String>) -> u64 {
    let value = raw.unwrap_or_else(|| {
        usage(program);
        panic!("{name} requires a value");
    });

    value.parse::<u64>().unwrap_or_else(|err| {
        usage(program);
        panic!("invalid {name} value '{value}': {err}");
    })
}

fn parse_usize_arg(program: &str, name: &str, raw: Option<String>) -> usize {
    let value = raw.unwrap_or_else(|| {
        usage(program);
        panic!("{name} requires a value");
    });

    value.parse::<usize>().unwrap_or_else(|err| {
        usage(program);
        panic!("invalid {name} value '{value}': {err}");
    })
}

fn parse_bench_args_impl() -> BenchArgs {
    let mut duration_seconds = env::var("BENCHMARK_DURATION_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_DURATION_SECONDS);
    let mut sender_accounts = env::var("BENCH_SENDER_ACCOUNTS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_SENDER_ACCOUNTS);
    let mut recipient_accounts = env::var("BENCH_RECIPIENT_ACCOUNTS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_RECIPIENT_ACCOUNTS);
    let mut block_interval_ms = env::var("BENCH_BLOCK_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_BLOCK_INTERVAL_MS);

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "single_node_transfer_benchmark".into());
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--duration-seconds" => {
                duration_seconds = parse_u64_arg(&program, "--duration-seconds", iter.next());
            }
            "--sender-accounts" => {
                sender_accounts = parse_usize_arg(&program, "--sender-accounts", iter.next());
            }
            "--recipient-accounts" => {
                recipient_accounts = parse_usize_arg(&program, "--recipient-accounts", iter.next());
            }
            "--block-interval-ms" => {
                block_interval_ms = parse_u64_arg(&program, "--block-interval-ms", iter.next());
            }
            "--whirlpool-node-bin" => {
                let _ignored = iter.next().unwrap_or_else(|| {
                    usage(&program);
                    panic!("--whirlpool-node-bin requires a path");
                });
            }
            "--help" | "-h" => {
                usage(&program);
                std::process::exit(0);
            }
            _ => {
                usage(&program);
                panic!("unknown argument: {arg}");
            }
        }
    }

    if duration_seconds == 0 {
        usage(&program);
        panic!("--duration-seconds must be > 0");
    }
    if sender_accounts == 0 {
        usage(&program);
        panic!("--sender-accounts must be > 0");
    }
    if recipient_accounts == 0 {
        usage(&program);
        panic!("--recipient-accounts must be > 0");
    }
    if block_interval_ms == 0 {
        usage(&program);
        panic!("--block-interval-ms must be > 0");
    }

    BenchArgs {
        duration_seconds,
        sender_accounts,
        recipient_accounts,
        block_interval_ms,
    }
}
