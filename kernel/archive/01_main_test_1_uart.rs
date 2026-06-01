#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Assembly entry point (_start)
// Initializes a stack, writes 'X' repeatedly to UART0, and calls into Rust once.
// -----------------------------------------------------------------------------
global_asm!(
    r#"
    .section .text._start, "ax"
    .global _start
_start:
    // Set up a simple stack
    ldr x0, =0x40080000
    mov sp, x0

    // UART0 base address
    ldr x1, =0x09000000

    // w0 = character 'X'
    mov w0, #'X'

    // Call into Rust once
    bl rust_main

1:  str w0, [x1]    // UART0_DR = 'X'
    b 1b            // infinite loop
"#
);

// -----------------------------------------------------------------------------
// Rust entry point
// Writes a single 'Z' to UART0 using a raw pointer store.
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn rust_main() {
    unsafe {
        let uart = 0x0900_0000 as *mut u32;
        // No write_volatile, no libcore calls — just a raw store
        *uart = b'Z' as u32;
    }
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
// These are needed because libcore expects memcpy/memcmp symbols to exist.
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
