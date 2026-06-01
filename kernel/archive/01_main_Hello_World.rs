#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Assembly entry point (_start)
// -----------------------------------------------------------------------------
global_asm!(
    r#"
    .section .text._start, "ax"
    .global _start
_start:
    // Set up a simple stack
    ldr x0, =0x40080000
    mov sp, x0

    // Jump into Rust
    bl rust_main

1:  b 1b   // Halt: infinite loop
"#
);

// -----------------------------------------------------------------------------
// Rust entry point
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn rust_main() {
    // C‑style null‑terminated string
    let msg = b"Hello from Rust kernel!\n\0";
    uart_puts(msg.as_ptr());
}

// -----------------------------------------------------------------------------
// Panic handler (required in no_std)
// -----------------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// -----------------------------------------------------------------------------
// Minimal C runtime shims required by libcore
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while i < n {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    }
    dst
}

#[no_mangle]
pub extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n {
            let da = *a.add(i);
            let db = *b.add(i);
            if da != db {
                return da as i32 - db as i32;
            }
            i += 1;
        }
    }
    0
}

// -----------------------------------------------------------------------------
// UART driver (MMIO at 0x0900_0000)
// -----------------------------------------------------------------------------
#[inline(always)]
fn uart_putc(c: u8) {
    unsafe {
        let uart = 0x0900_0000 as *mut u32;
        *uart = c as u32;
    }
}

#[inline(always)]
fn uart_puts(s: *const u8) {
    unsafe {
        let mut p = s;
        loop {
            let c = *p;
            if c == 0 {
                break;
            }
            uart_putc(c);
            p = p.add(1);
        }
    }
}
