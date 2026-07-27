// src/drivers/uart.rs

// -----------------------------------------------------------------------------
// UART driver
// -----------------------------------------------------------------------------

use crate::arch::mmio::UART_BASE;

#[inline(always)]
pub fn putc(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART_BASE as *mut u8, c);
    }
}

pub fn puts(s: &str) {
    for &b in s.as_bytes() {
        putc(b);
    }
}

#[inline(always)]
pub fn getc() -> u8 {
    unsafe {
        core::ptr::read_volatile(UART_BASE as *const u8)
    }
}
