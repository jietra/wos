#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

mod arch;
mod drivers;
mod memory;
mod mmu;
mod utils; // required for shims C-implemented functions like puts, put_hex_ln, etc.
//mod debug;

use arch::aarch64::init_exceptions;
use arch::aarch64::linker_symbols::_kernel_end;
use drivers::uart::puts;
use memory::phys::{init_phys_alloc, alloc_page};
use mmu::{init_mair, init_tcr, init_ttbr0, enable_mmu, init_page_tables};

// -----------------------------------------------------------------------------
// Assembly entry point (_start)
// -----------------------------------------------------------------------------
global_asm!(include_str!("arch/aarch64/start.S"));
global_asm!(include_str!("arch/aarch64/exception_vectors.S"));  // Let the linker know about the exception vectors (required for interrupts, etc.)

// -----------------------------------------------------------------------------
// Rust entry point
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn rust_main() {
    puts("Booting WOS...\n");

    unsafe {
        init_exceptions();  // Initialize exception vectors
        init_mair();        // Initialize MAIR (Memory Attribute Indirection Register) to set up memory attributes
        init_tcr();         // Initialize TCR (Translation Control Register) to set up the virtual address space size and granule size
        init_page_tables();
        init_ttbr0();
        enable_mmu();
        core::arch::asm!("isb"); // Ensure that all changes to the MMU configuration are visible before we continue
        init_phys_alloc(&_kernel_end as *const u8 as u64);
    }

    //unsafe { debug::memory::test_memory(); }

    // C‑style null‑terminated string
    puts("\nHello from WOS!\n\n");
}

// -----------------------------------------------------------------------------
// Panic handler (required in no_std)
// -----------------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}