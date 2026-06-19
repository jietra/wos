// src/time/tick.rs

use core::sync::atomic::{AtomicU64, Ordering};

pub static TICK: AtomicU64 = AtomicU64::new(0);

pub fn tick_now() -> u64 {
    TICK.load(Ordering::Relaxed)
}

pub fn on_tick() {
    TICK.fetch_add(1, Ordering::Relaxed);
}
