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

use crate::scheduler::vm::{
    init_guest,
    load_guest,
    run_vm
};
use crate::arch::vm_if::VmArch;
use crate::arch::ArchVm;

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

// --------------------
// INIT ARCH EL2
// --------------------

pub fn init_arch_el2() {
    crate::uart_println!("\n===== Initializing arch EL2... =====\n");

    crate::uart_println!("Initializing stage-2 MMU...");
    unsafe {
        // 1. init VTCR_EL2
        mmu::stage2::init_vtcr();

        // 2. init MAIR_EL2: normal memory, WB, cacheable, index 0
        mmu::stage2::init_mair();

        // 3. init VTTBR_EL2: root table
        mmu::stage2::init_vttbr();
        
        // 4. init S2 table pool
        mmu::stage2::init_s2_table_pool();

        // 5. init hypervisor
        crate::uart_println!("Initializing hypervisor with simple guest...");
        let ipa_base = 0x4000_0000;
        let ipa_size = 64 * 1024 * 1024;
        //let ipa_size = ((GUEST_BIN.len() + 0xFFF) & !0xFFF) as u64;
        let uart_ipa = 0x0900_0000;
        let uart_pa  = 0x0900_0000;
        crate::uart_println!("\tipa_base = ", ipa_base);
        crate::uart_println!("\tipa_size = ", ipa_size);

        init_guest(ipa_base);
        load_guest(0, ipa_base, ipa_size, ipa_base, uart_ipa, uart_pa);

        // 6. Activate virt (HCR_EL2)
        mmu::stage2::init_hcr();

        // |CHECK| MMU EL2 checks
        crate::uart_println!("|CHECK| MMU EL2 params:");

        let mut vtcr: u64;
        let mut vttbr: u64;
        let mut hcr: u64;
        core::arch::asm!("mrs {}, vtcr_el2", out(reg) vtcr);
        core::arch::asm!("mrs {}, vttbr_el2", out(reg) vttbr);
        core::arch::asm!("mrs {}, hcr_el2", out(reg) hcr);

        crate::uart_println!("\tVTCR_EL2  = 0x{:016x}", vtcr);
        crate::uart_println!("\tVTTBR_EL2 = 0x{:016x}", vttbr);
        crate::uart_println!("\tHCR_EL2   = 0x{:016x}", hcr);
        crate::uart_println!("\tS2_ROOT   = 0x{:016x}", ArchVm::get_s2_root());
        let root_ptr = ArchVm::get_s2_root_entries();
        crate::uart_println!("\tS2_ROOT[0]= 0x{:016x}", *root_ptr.add(0));
        crate::uart_println!("\tS2_ROOT[1]= 0x{:016x}", *root_ptr.add(1));

        // 7. TLB flush
        crate::uart_println!("Flushing TLB...");
        core::arch::asm!(
            "dsb ish",
            "tlbi alle2",
            "dsb ish",
            "isb",
        );
        crate::uart_println!("\tTLB flushed.");

        // 8. init guest stack
        crate::uart_println!("Initializing sp_el1...");
        let guest_stack_ipa  = 0x0000_0000_0400_0100;//0x40080000;//0x4000_8000;
        let guest_stack_size = 0x10000; // 64 KiB
        let sp_el1 = guest_stack_ipa + guest_stack_size - 16;
        crate::uart_println!("\tsp_el1 = 0x{:016x}", sp_el1);
        core::arch::asm!(
            "msr sp_el1, {0}",
            in(reg) sp_el1,
        );
        // Check
        let sp: u64;
        core::arch::asm!("mrs {0}, sp_el1", out(reg) sp);
        crate::uart_println!("\tSP_EL1 = 0x{:016x}", sp);
    }

    // --- Welcome message --------------------------------
    puts("\n\n==========================================================\n");
    puts("                   xWALT Hypervisor v0.1\n");
    puts("                     (c) 2026 Ulrich Tan\n");
    puts("==========================================================\n\n");

    puts("[ OK ] EL2 mode entered\n");
    puts("[ OK ] UART initialized\n");
    puts("[ OK ] EL2 MMU (Stage2)\n");
    puts("[ OK ] EL2 exception vectors\n");
    puts("[ .. ] No EL2 interrupt controller\n\n");

    puts("Booting guest kernel...\n\n");

    puts("       ██╗    ██╗ █████╗ ██╗   ████████╗\n");
    puts("       ██║    ██║██╔══██╗██║   ╚══██╔══╝\n");
    puts("██╗    ██║ █╗ ██║███████║██║      ██║   \n");
    puts("█████╗ ██║███╗██║██╔══██║██║      ██║   \n");
    puts("██║ ██║╚███╔███╔╝██║  ██║███████╗ ██║   \n");
    puts("╚═╝ ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝ ╚═╝   \n\n");
    puts("        hWALT | xWALT HYPERVISOR        \n\n");

    puts(" ---------------------------------------\n");
    puts(" |        Hello from hWALT-EL2!        |\n");
    puts(" ---------------------------------------\n\n");

    crate::uart_println!("===== Arch EL2 initialized. =====\n");
    
    unsafe {
        run_vm(0)
    }
}