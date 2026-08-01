// src/scheduler/vm.rs

use crate::arch::vm_if::VmArch;
use crate::arch::ArchVm;

// Guests
pub static GUEST_BIN: &[u8] = include_bytes!("../../../guest/guest.bin");
#[no_mangle]
pub static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];
pub const MAX_VMS: usize = 4;

/// VM struct
#[derive(Copy, Clone)]
pub struct VM {
    pub id: usize,
    pub ipa_base: u64,     // Guest IPA base
    pub ipa_size: u64,     // Guest memory size
    pub entry: u64,        // Guest entry point
    pub s2_root: u64,      // Stage-2 root table (VTTBR_EL2)
}

/// Initialize first guest
pub unsafe fn init_guest(ipa_base: u64) {
    crate::uart_println!("Initializing guest...");
    
    // put guest in ram
    crate::uart_println!("\tPut guest binary in ram @ipa_base...");
    let dst = ipa_base as *mut u8;
    for i in 0..GUEST_BIN.len() {
        core::ptr::write_volatile(dst.add(i), GUEST_BIN[i]);
    }

    // Check : read 16 first bytes
    crate::uart_println!("\t|CHECK| Guest first 16 bytes:");
    for i in 0..16 {
        let b = core::ptr::read_volatile(dst.add(i));
        crate::uart_println!("\t  [{:02}] = 0x{:02x}", i, b);
    }
    let entry = ipa_base as *const u32;
    crate::uart_println!("\t|CHECK| Guest entry words:");
    for i in 0..4 {
        let w = core::ptr::read_volatile(entry.add(i));
        crate::uart_println!("\t  [{:02}] = 0x{:08x}", i, w);
    }

    crate::uart_println!("\tGuest initialized.");
}

/// Load guest from id, ipa_base, ipa_size and entry
pub unsafe fn load_guest(id: usize, ipa_base: u64, ipa_size: u64, entry: u64, uart_ipa: u64, uart_pa: u64) {
    crate::uart_println!("Loading guest...");

    // map IPA -> PA identity
    crate::uart_println!("\tMapping kernel ipa to pa...");
    for off in (0..ipa_size).step_by(4096) {
        ArchVm::map_page(ipa_base + off, ipa_base + off);
    }

    crate::uart_println!("\tRegister guest in VMs table...");
    VMS[id] = Some(VM {
        id,
        ipa_base,
        ipa_size,
        entry,
        s2_root: ArchVm::get_s2_root(),
    });

    crate::uart_println!("\tMapping uart ipa to pa...");
    crate::arch::ArchVm::map_device(uart_ipa, uart_pa);

    crate::uart_println!("\tGuest loaded.");
}

/// Run VM from id
pub unsafe fn run_vm(id: usize) -> ! {
    crate::uart_println!("Running VM...");
    
    // unwrap VM from VMS table
    crate::uart_println!("\tUnwrap VM...");
    let vm = VMS[id].unwrap();
    crate::uart_println!("\t\tVM entry = 0x{:016x}", vm.entry);

    // Reset EL1 context
    crate::uart_println!("\tReset EL1 context...");
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
    
    crate::uart_println!("\t\tspsr     = 0x{:016x}", spsr);

    // Launch VM...
    crate::uart_println!("\tRunning VM...\n");

    core::arch::asm!(
        "msr elr_el2, {entry}",
        "msr spsr_el2, {psr}",
        "eret",
        entry = in(reg) vm.entry,
        psr = in(reg) spsr,         // EL1h
    );

    loop {}
}