// src/scheduler/vm.rs

use crate::arch::vm_if::VmArch;
use crate::arch::ArchVm;

#[derive(Copy, Clone)]
pub struct VM {
    pub id: usize,
    pub ipa_base: u64,     // Guest IPA base
    pub ipa_size: u64,     // Guest memory size
    pub entry: u64,        // Guest entry point
    pub s2_root: u64,      // Stage-2 root table (VTTBR_EL2)
}

pub const MAX_VMS: usize = 4;

#[no_mangle]
pub static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];

pub unsafe fn load_guest(id: usize, ipa_base: u64, ipa_size: u64, entry: u64) {
    crate::uart_println!("Loading guest...");
    // map IPA -> PA identity
    crate::uart_println!("\tMapping ipa to pa...");
    for off in (0..ipa_size).step_by(4096) {
        ArchVm::map_page(ipa_base + off, ipa_base + off);
    }

    crate::uart_println!("\tRegister VM...");
    VMS[id] = Some(VM {
        id,
        ipa_base,
        ipa_size,
        entry,
        s2_root: ArchVm::get_s2_root(),
    });

    crate::uart_println!("\tGuest loaded.");
}

pub unsafe fn run_vm(id: usize) -> ! {
    crate::uart_println!("Running VM...");
    let vm = VMS[id].unwrap();
    crate::uart_println!("VM entry = 0x{:016x}", vm.entry);

    // Reset EL1 context
    let zero: u64 = 0;
    core::arch::asm!(
        "msr ttbr0_el1, {0}",
        "msr ttbr1_el1, {0}",
        "msr tcr_el1,   {0}",
        "msr mair_el1,  {0}",
        "msr sctlr_el1, {0}",   // MMU off, caches off
        "isb",
        in(reg) zero,
    );

    let spsr = 
        (0b0101 << 0) |   // M[3:0] = 0101 → EL1h
        (1 << 6)      |   // F = 1 (FIQ masked)
        (1 << 7)      |   // I = 1 (IRQ masked)
        (1 << 8);         // A = 1 (SError masked)
    
    crate::uart_println!("spsr = 0x{:016x}", spsr);

    core::arch::asm!(
        "msr elr_el2, {entry}",
        "msr spsr_el2, {psr}",
        "eret",
        entry = in(reg) vm.entry,
        psr = in(reg) spsr,         // EL1h
    );

    loop {}
}
