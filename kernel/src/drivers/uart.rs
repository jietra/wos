// -----------------------------------------------------------------------------
// UART driver (MMIO at 0x0900_0000)
// -----------------------------------------------------------------------------

#[inline(always)]
pub fn putc(c: u8) {
    unsafe {
        let uart = 0x0900_0000 as *mut u32;
        *uart = c as u32;
    }
}

pub fn puts(s: &str) {
    for &b in s.as_bytes() {
        putc(b);
    }
}