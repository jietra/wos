#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

mod arch;
mod drivers;
mod memory;
mod mmu;
mod utils; // required for include shims C-implemented functions enabling puts, put_hex_ln, etc.

mod debug;

use arch::aarch64::init_exceptions;
use arch::aarch64::linker_symbols::_kernel_end; // defined in linker script: required for initializing physical memory allocator
use drivers::uart::puts;
use memory::phys::init_phys_alloc;
use mmu::{init_mair, init_tcr, init_ttbr0, enable_mmu, init_page_tables};
use utils::print::put_hex_ln;

// -----------------------------------------------------------------------------
// Assembly entry point (_start and exception vectors))
// -----------------------------------------------------------------------------
global_asm!(include_str!("arch/aarch64/start.S"));
global_asm!(include_str!("arch/aarch64/exception_vectors.S"));  // Let the linker know about the exception vectors (required for interrupts, etc.)

// -----------------------------------------------------------------------------
// Rust entry point
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn rust_main() {
    puts("Booting WOS...\n");

    // --- Enable FP/SIMD in CPACR_EL1 to allow use of floating-point and SIMD instructions ---
    // Required for codes such as read_volatile etc.
    // (LLVM may indeed generate FP/SIMD instructions for some operations,
    // even if we don't explicitly use them in our Rust code - e.g. 
    // for mathematical operations, or even for some memory access patterns).
    unsafe fn enable_fp() {
        core::arch::asm!(
            "mrs x0, CPACR_EL1",
            "orr x0, x0, #(0b11 << 20)", // FPEN = 0b11 (enable FP/SIMD)
            "msr CPACR_EL1, x0",
            "isb",
            options(nostack, preserves_flags),
        );
    }
    unsafe { enable_fp(); }

    // | DEBUG | Reading current EL --------------------------------
    unsafe { debug::debug::read_current_el(); }

    // --- Initializing exception vectors --------------------------------
    puts("Initializing exception vectors...\n");
    unsafe {
        init_exceptions();      // install VBAR_EL1 right away
    }
    
    // | DEBUG | Reading and parsing the DTB --------------------------------
    puts("| DEBUG | Reading DTB...\n");
    unsafe {
        debug::dtb::debug_dtb();
        //debug::dtb::parse_dtb();
    }

    // | DEBUG | TESTING SEQUENCE --------------------------------
    puts("| DEBUG | Running some tests...\n");

    puts("\tTesting read_volatile...\n");
    static TEST: u64 = 0x12345678ABCDEF00;
    extern "C" {
        fn _start();
    }
    unsafe {
        puts("\t\taddr(TEST) \t= "); put_hex_ln(&TEST as *const _ as u64);
        let v = core::ptr::read_volatile(&TEST);
        puts("\t\tr_vol(TEST) \t= "); put_hex_ln(v);
        let va = _start as *const () as u64;
        puts("\t\tVA(_start) \t= "); put_hex_ln(va);
        let pa: u64;
        core::arch::asm!("adrp {0}, _start", out(reg) pa);
        puts("\t\tPA(_start) \t= "); put_hex_ln(pa);
        let p = va as *const u32;
        let w = core::ptr::read_volatile(p) as u64;
        puts("\t\tr_vol(_start) \t= "); put_hex_ln(w);
    }

    puts("\tTesting BRK instruction and exception handling...\n");
    let spsel: u64;
    unsafe {
        core::arch::asm!("mrs {0}, SPSel", out(reg) spsel);
    }
    puts("\t\tSPSel \t= 0x"); put_hex_ln(spsel);
    let esr: u64;
    let far: u64;
    unsafe {
        core::arch::asm!(
            "mrs {0}, ESR_EL1",
            "mrs {1}, FAR_EL1",
            out(reg) esr,
            out(reg) far,
            options(nostack, preserves_flags),
        );
    }
    puts("\t\tESR_EL1 = 0x"); put_hex_ln(esr);
    puts("\t\tFAR_EL1 = 0x"); put_hex_ln(far);
    /*
    puts("\t\t----- BREAK... -----\n");
    unsafe {
        core::arch::asm!("brk #0");
    }
    puts("\t\tAfter BRK\n");
    */

    // --- Initializing MMU and page tables --------------------------------
    puts("Initializing MMU...\n");
    unsafe {
        init_mair();                // Initialize MAIR (Memory Attribute Indirection Register) to set up memory attributes
        init_tcr();                 // Initialize TCR (Translation Control Register) to set up the virtual address space size and granule size
        init_page_tables();
        init_ttbr0();
        enable_mmu();
        core::arch::asm!("isb");    // Ensure that all changes to the MMU configuration are visible before we continue
        init_phys_alloc(&_kernel_end as *const u8 as u64);
    }
    puts("\tMMU enabled\n");

    // | DEBUG | Testing memory access after MMU enabled --------------------------------
    puts("| DEBUG | Testing memory access after MMU enabled...\n");
    unsafe { debug::memory::test_memory(); }
    
    // --- Welcome message --------------------------------
    puts("\n-------------------------------\n");
    puts(  "|       Hello from WOS!       |"  );
    puts("\n-------------------------------\n");
}

// -----------------------------------------------------------------------------
// Panic handler (required in no_std)
// -----------------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}