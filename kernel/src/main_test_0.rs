#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Minimal test: write 'X' repeatedly to UART0 (MMIO at 0x0900_0000)
// -----------------------------------------------------------------------------
global_asm!(
    r#"
    .global _start
_start:
    // x1 = UART0 base address
    ldr x1, =0x09000000

    // w0 = character 'X'
    mov w0, #'X'

1:
    str w0, [x1]   // UART0_DR = 'X'
    b 1b           // infinite loop
"#
);

// Required by no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
