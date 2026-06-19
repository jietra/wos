// arch/aarch64/mod.rs

pub mod boot;
pub mod mmio;
pub mod cpu;
pub mod mmu;
pub mod gic;
pub mod timer;
pub mod irq;
pub mod uart;

use crate::drivers::uart::puts;
use crate::memory::phys::init_phys_alloc;
use boot::linker_symbols::_kernel_end; // defined in linker script: required for initializing physical memory allocator
use cpu::exceptions::init_exceptions;
use mmu::{init_mair, init_tcr, init_ttbr0, enable_mmu, init_page_tables};
use gic::gicv2::gicv2;

pub fn init_arch() {
    puts("| BOOT  | Booting WOS...\n");

    // | CHECK | CPU checks  --------------------------------
    unsafe {
        crate::uart_println!("| CHECK | CPU checks...");
        crate::debug::cpu::read_current_el();   // Reading current EL
        crate::debug::cpu::dump_mpidr();        // Reading current CPU
        crate::debug::cpu::read_daif();         // Reading DAIF to check whether IRQ are unmasked after boot
    }

    // --- Initializing exception vectors --------------------------------
    puts("| INIT. | Initializing exception vectors...\n");
    unsafe { init_exceptions(); }     // install VBAR_EL1 right away

    // | CHECK | Reading and parsing the DTB --------------------------------
    unsafe {
        crate::debug::dtb::debug_dtb();
        //crate::debug::dtb::parse_dtb();
    }

    // | CHECK | TESTING SEQUENCE --------------------------------
    unsafe {
        crate::debug::tests::tests();
        //crate::debug::tests::test_break();
    }    
/*
    // --- Initializing MMU and page tables --------------------------------
    puts("| INIT. | Initializing MMU...\n");
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

    // | CHECK | Testing memory access after MMU enabled --------------------------------
    unsafe { crate::debug::memory::test_memory(); }
*/

    // --- Initializing Gicv2 -----------------------------
    puts("| INIT. | Initializing GIC v2...\n");
    unsafe { 
        gicv2::init();
        gicv2::dump_gic();
    }
    puts("\tGIC enabled\n");

    puts("\n==========================================================\n");

    puts("\nWOS-AARCH64 Firmware v0.1\n");
    puts("(c) 2026 Ulrich Tan\n\n");

    puts("[ OK ] CPU initialized\n");
    puts("[ OK ] Exception vectors initialized\n");
    puts("[ OK ] MMU initialized\n");
    puts("[ OK ] GICv2 initialized\n");
    puts("[ OK ] UART ready\n\n");

    puts("Booting kernel...\n\n");

    puts("██╗    ██╗ ██████╗  ██████╗\n");
    puts("██║    ██║██╔═══██╗██╔════╝\n");
    puts("██║ █╗ ██║██║   ██║ █████╗ \n");
    puts("██║███╗██║██║   ██║     ██║\n");
    puts("╚███╔███╔╝╚██████╔╝██████╔╝\n");
    puts(" ╚══╝╚══╝  ╚═════╝ ╚═════╝ \n\n");
    puts(" W O S   –   A A R C H 6 4\n\n");

    // --- Welcome message --------------------------------
    puts("\n---------------------------------------\n");
    puts(  "|       Hello from WOS-AARCH64!       |"  );
    puts("\n---------------------------------------\n\n");

    // | CHECK | Sending an SGI "this CPU only" ---------------------
    unsafe {
        irq::debug_irq::sgi_irq();
    }

    crate::uart_println!("\n");

    // --- Call timer -------------------------------------
    unsafe { crate::arch::aarch64::timer::cntp::cntp::init(); }

}