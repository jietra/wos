#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Assembly entry point: read CurrentEL and print it on UART0
// -----------------------------------------------------------------------------
global_asm!(
    r#"
    .section .text._start, "ax"
    .global _start
_start:
    // UART0 base address
    ldr x1, =0x09000000

    // Read CurrentEL
    mrs x0, CurrentEL

    // CurrentEL format:
    //   bits [3:2] = EL number
    //   e.g. 0x8 = EL2, 0x4 = EL1
    // Shift right by 2 to get the raw EL number
    lsr x0, x0, #2

    // Convert to ASCII: '0' + EL
    add x0, x0, #'0'

    // Write to UART
    str w0, [x1]

1:  b 1b
"#
);

// Required by no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
