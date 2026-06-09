// -----------------------------------------------------------------------------
// UART driver
// -----------------------------------------------------------------------------

use crate::arch::mmio::UART_BASE;

#[inline(always)]
pub fn putc(c: u8) {
    unsafe {
        let uart = UART_BASE as *mut u8;
        core::ptr::write_volatile(uart, c);
    }
}

pub fn puts(s: &str) {
    for &b in s.as_bytes() {
        putc(b);
    }
}