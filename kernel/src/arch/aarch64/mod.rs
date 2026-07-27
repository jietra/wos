// arch/aarch64/mod.rs

pub mod boot;
pub mod mmio;
pub mod cpu;
pub mod svc;
pub mod mmu;
pub mod gic;
pub mod timer;
pub mod irq;
pub mod uart;
pub mod process;
pub mod syscall;

use crate::drivers::uart::puts;
use crate::memory::phys::init_phys_alloc;
use boot::linker_symbols::_kernel_end;      // defined in linker script: required for initializing physical memory allocator
use cpu::exceptions::init_exceptions;
use mmu::{
    init_mair, 
    init_tcr, 
    init_ttbr0, 
    init_ttbr1, 
    enable_mmu, 
    init_page_tables, 
    set_vbar_in_va
};
use gic::gicv2::gicv2;

pub fn init_arch() {
    puts("| BOOT  | Booting xWALT...\n");

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
    crate::debug::tests::tests();
    //unsafe { crate::debug::tests::test_break(); }    

    // --- Initializing MMU and page tables --------------------------------
    puts("| INIT. | Initializing MMU...\n");
    unsafe {
        init_mair();                // Initialize MAIR (Memory Attribute Indirection Register) to set up memory attributes
        
        init_tcr();                 // Initialize TCR (Translation Control Register) to set up the virtual address space size and granule size
        
        // | CHECK | linker addresses ---------------------
        crate::uart_println!("\tcheck linker addresses pre-MMU...");
        use boot::linker_symbols::_text_start;
        use boot::linker_symbols::_text_end;
        use boot::linker_symbols::_stack_start;
        use boot::linker_symbols::_stack_top;
        use boot::linker_symbols::_exceptions_start;
        use boot::linker_symbols::_exceptions_end;
        let text_start = &_text_start       as *const u8 as u64;
        let text_end   = &_text_end         as *const u8 as u64;
        let stack_start= &_stack_start      as *const u8 as u64;
        let stack_top  = &_stack_top        as *const u8 as u64;
        let exc_start  = &_exceptions_start as *const u8 as u64;
        let exc_end    = &_exceptions_end   as *const u8 as u64;
        crate::uart_println!("\t\t_text_start  = 0x{:016x}", text_start);
        crate::uart_println!("\t\t_text_end    = 0x{:016x}", text_end);        
        crate::uart_println!("\t\t_stack_start = 0x{:016x}", stack_start);
        crate::uart_println!("\t\t_stack_top   = 0x{:016x}", stack_top);
        crate::uart_println!("\t\t_exc_start   = 0x{:016x}", exc_start);
        crate::uart_println!("\t\t_exc_end     = 0x{:016x}", exc_end);

        init_ttbr0();
        
        init_ttbr1();

        init_phys_alloc(&_kernel_end as *const u8 as u64);

        init_page_tables();

        enable_mmu();
        core::arch::asm!("isb");    // Ensure that all changes to the MMU configuration are visible before we continue

        crate::uart_println!("\t---\n\tsetting VBAR_EL1 to virtual address...");
        set_vbar_in_va();           // set VBAR in virtual adress (since MMU now activated)
        
    }

    // | CHECK | Testing memory access after MMU enabled --------------------------------
    unsafe { crate::debug::memory::test_memory(); }


    // --- Initializing Gicv2 -----------------------------
    puts("| INIT. | Initializing GIC v2...\n");
    unsafe {         
        gicv2::init();
        gicv2::dump_gic();
        
        crate::uart_println!("\t---");
        use crate::arch::gic::gicv2::gicv2::GICD_VADDR;
        let d_ctlr_va = crate::arch::gic::gicv2::gicv2::mmio_read32(GICD_VADDR + 0x000);
        crate::uart_println!("\tGICD_CTLR VA  = 0x{:08x}", d_ctlr_va);
    }
    puts("\tGIC enabled\n");

    // | CHECK | Sending an SGI "this CPU only" ---------------------
    unsafe { irq::debug_irq::sgi_irq(); }

    // --- Initializing timer -------------------------------------
    unsafe { crate::arch::aarch64::timer::cntp::cntp::init(); }

    // // Enable timer IRQ in GIC (it is actually a redundancy) ------------------------
    unsafe { crate::arch::aarch64::gic::gicv2::gicv2::enable_irq(crate::arch::aarch64::timer::cntp::cntp::TIMER_IRQ); }
    
    puts("\n\n==========================================================\n");
    puts(    "                xWALT-AARCH64 Firmware v0.1               \n");
    puts(    "                    (c) 2026 Ulrich Tan                   \n");
    puts(    "==========================================================\n\n");

    puts("[ OK ] CPU initialized\n");
    puts("[ OK ] Exception vectors initialized\n");
    puts("[ OK ] MMU initialized\n");
    puts("[ OK ] GICv2 initialized\n");
    puts("[ OK ] UART ready\n\n");

    puts("Booting kernel...\n\n");

    puts("       ██╗    ██╗ █████╗ ██╗  ████████╗\n");
    puts("       ██║    ██║██╔══██╗██║  ╚══██╔══╝\n");
    puts("██╗ ██╗██║ █╗ ██║███████║██║     ██║   \n");
    puts(" ╚██╔═╝██║███╗██║██╔══██║██║     ██║   \n");
    puts("██║ ██║╚███╔███╔╝██║  ██║███████╗██║   \n");
    puts("╚═╝ ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝╚═╝   \n\n");
    puts("           xWALT OS – AARCH64          \n\n");

    // --- Welcome message --------------------------------
    puts("\n-----------------------------------------\n");
    puts(  "|       Hello from xWALT-AARCH64!       |"  );
    puts("\n-----------------------------------------\n\n");

    // | CHECK | Init and launch 3 tasks --------------------------------
    puts("xWALT OS is ready.\n\n");
    unsafe { crate::arch::aarch64::timer::cntp::cntp::disable_cntp(); }
    unsafe { ask_launch_demo_tasks(); }

}

use crate::drivers::uart::getc;
pub fn ask_launch_demo_tasks() {
    puts("Launching demo processe(s) ? [Y/N]\n");

    loop {
        let c = getc(); // reads UART

        match c {
            b'Y' | b'y' => {
                puts("-> Starting demo tasks...\n");
                //puts("enable cntp:");
                //unsafe { crate::arch::aarch64::timer::cntp::cntp::enable_cntp(); }
                puts("init processes:");
                unsafe { crate::arch::aarch64::process::process::init_processes(); }
                puts("start first process:");
                unsafe { crate::arch::aarch64::process::process::start_first_proc_rust(); }
                break;
            }
            b'N' | b'n' => {
                puts("-> Skipping demo tasks.\n");
                break;
            }
            _ => {
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main_el2() -> ! {
    // init minimal hypervisor: log, setup, boucle, etc.
    // --- Welcome message --------------------------------
    puts("\n\n==========================================================\n");
    puts("                   xWALT Hypervisor v0.1\n");
    puts("                     (c) 2026 Ulrich Tan\n");
    puts("==========================================================\n\n");

    puts("[ OK ] EL2 mode entered\n");
    puts("[ OK ] UART initialized\n");
    puts("[ .. ] No EL2 MMU (VM=0)\n");
    puts("[ .. ] No EL2 exception vectors\n");
    puts("[ .. ] No EL2 interrupt controller\n\n");

    puts("Booting guest kernel...\n\n");

    puts("        ██╗    ██╗ █████╗ ██╗   ████████╗\n");
    puts("        ██║    ██║██╔══██╗██║   ╚══██╔══╝\n");
    puts("██╗  ██╗██║ █╗ ██║███████║██║      ██║   \n");
    puts(" ╚███╔╝ ██║███╗██║██╔══██║██║      ██║   \n");
    puts("██║  ██║╚███╔███╔╝██║  ██║███████╗ ██║   \n");
    puts("╚═╝  ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝ ╚═╝   \n\n");
    puts("              xWALT HYPERVISOR           \n\n");

    puts("---------------------------------------\n");
    puts("|        Hello from xWALT-EL2!        |\n");
    puts("---------------------------------------\n\n");

    loop{};
}
