#![allow(dead_code)]

use std::net::TcpListener;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::OnceLock;

static NEXT_PORT: OnceLock<AtomicU16> = OnceLock::new();

fn next_port_counter() -> &'static AtomicU16 {
    NEXT_PORT.get_or_init(|| {
        let start = TcpListener::bind("127.0.0.1:0")
            .expect("failed to bind initial ephemeral port")
            .local_addr()
            .expect("failed to read initial ephemeral port")
            .port();
        AtomicU16::new(start)
    })
}

pub(crate) fn allocate_port() -> u16 {
    let counter = next_port_counter();
    for _ in 0..2048 {
        let candidate = counter.fetch_add(1, Ordering::SeqCst);
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", candidate)) {
            drop(listener);
            return candidate;
        }
    }

    panic!("failed to allocate a free localhost port after retries");
}
