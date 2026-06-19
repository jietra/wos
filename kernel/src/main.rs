#![no_std]
#![no_main]

mod arch;
mod drivers;
mod utils; // required for include shims C-implemented functions enabling puts, put_hex_ln, etc.
mod memory;
mod debug;
mod time;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    arch::init_arch();
    loop {}
}

// -----------------------------------------------------------------------------
// Panic handler (required in no_std)
// -----------------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
