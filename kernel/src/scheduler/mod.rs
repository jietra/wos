// src/scheduler/mod.rs

use core::sync::atomic::{AtomicUsize, Ordering};

static CURRENT: AtomicUsize = AtomicUsize::new(0);

pub const NUM_TASKS: usize = 3;

pub fn on_tick() {
    let cur = CURRENT.load(Ordering::Relaxed);
    let next = (cur + 1) % NUM_TASKS;
    CURRENT.store(next, Ordering::Relaxed);

    let tick = crate::time::tick::tick_now();
    if tick % 100 == 0 {   // log only every 100 IRQs
        crate::uart_println!("[sched] switch to task {}", next);
    }
}

pub fn current() -> usize {
    CURRENT.load(Ordering::Relaxed)
}
