// src/scheduler/vm.rs

use crate::arch::vm_if::VmArch;
use crate::arch::ArchVm;
use crate::config::{
    VM_CONFIGS,
    VM_LINUX,
    LINUX_ENTRY_IPA,
    GUEST1_IPA_SIZE,
    GUEST1_IPA_BASE,
    GUEST1_PA_BASE,
    GUEST_UART_IPA,
    GUEST_GICD_IPA,
    GUEST_GICC_IPA,
    GUEST1_INITRAMFS_DATA,
    GUEST1_INITRAMFS_SIZE,
    GUEST1_KERNEL,
    GUEST1_DTB,
    GUEST1_KERNEL_SIZE,
    GUEST1_DTB_SIZE
};

// Guests
pub static GUEST_BIN: &[u8] = include_bytes!("../../../guest/guest.bin");
#[no_mangle]
pub static mut VMS: [Option<VM>; MAX_VMS] = [None; MAX_VMS];
pub const MAX_VMS: usize = 4;

/// VM struct
#[derive(Copy, Clone)]
pub struct VM {
    pub id: usize,
    pub ipa_base: u64,              // Guest IPA base
    pub ipa_size: u64,              // Guest memory size
    pub entry: u64,                 // Guest entry point
    pub s2_root: u64,               // Stage-2 root table (VTTBR_EL2)
    //pub dtb_ipa: Option<u64>,     // Device Tree Blob IPA (if any)
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

fn copy_blob_to_guest(guest_pa_base: u64, guest_ipa_base: u64, guest_ipa: u64, blob: &[u8]) {
    let offset = guest_ipa - guest_ipa_base;
    let dst = (guest_pa_base + offset) as *mut u8;
    for i in 0..blob.len() {
        unsafe { core::ptr::write_volatile(dst.add(i), blob[i]); }
    }
    crate::uart_println!("\t\tBlob copied to guest: dst 0x{:016x} -> dst+size 0x{:016x}",
        dst, dst.wrapping_add(blob.len())
    );
}

unsafe fn dump_binary_header(ipa: u64) {
    let base = (GUEST1_PA_BASE + (ipa - GUEST1_IPA_BASE)) as *const u8;
    crate::uart_println!("\t\t--- Dump binary ---");
    for i in 0..4 {
        let b = core::ptr::read_volatile(base.add(i));
        crate::uart_println!("\t\t{:02x} ", b);
        if (i + 1) % 16 == 0 {
            crate::uart_println!("");
        }
    }
    crate::uart_println!("\t\t-------------------");
}

pub unsafe fn load_linux_guest() -> u64 {
    let cfg = &VM_CONFIGS[VM_LINUX];

    // 1) Copy kernel image to guest memory
    crate::uart_println!("\tCopying kernel image to guest memory...");
    copy_blob_to_guest(
        GUEST1_PA_BASE, 
        GUEST1_IPA_BASE, 
        LINUX_ENTRY_IPA, 
        cfg.image_kernel
    );

    // 2) Copy DTB to guest memory
    let dtb_ipa  = GUEST1_IPA_BASE + 0x0200_0000; // +32MiB
    crate::uart_println!("\tCopying DTB to guest memory...");
    copy_blob_to_guest(
        GUEST1_PA_BASE,
        GUEST1_IPA_BASE,
        dtb_ipa,
        cfg.dtb
    );

    // |CHECK| no overlap
    crate::uart_println!("\t| CHECK | no overlap");
    let kernel_start_ipa = LINUX_ENTRY_IPA;
    let kernel_end_ipa   = LINUX_ENTRY_IPA + cfg.image_kernel.len() as u64;
    let dtb_start_ipa = dtb_ipa;
    let dtb_end_ipa   = dtb_ipa + cfg.dtb.len() as u64;
    crate::uart_println!(
        "\t\tkernel IPA: [0x{:016x} - 0x{:016x}]",
        kernel_start_ipa, kernel_end_ipa
    );
    crate::uart_println!(
        "\t\tdtb IPA:    [0x{:016x} - 0x{:016x}]",
        dtb_start_ipa, dtb_end_ipa
    );

    // |CHECK| dumps
    crate::uart_println!("\t| CHECK | kernel dump");
    dump_binary_header(LINUX_ENTRY_IPA);
    crate::uart_println!("\t\tKERNEL first bytes");
    for i in 0..4 { crate::uart_println!("\t\t{:02x}", GUEST1_KERNEL[i]); }
    crate::uart_println!("\t\tKERNEL SIZE = {}", GUEST1_KERNEL_SIZE);

    crate::uart_println!("\t| CHECK | dtb dump");
    dump_binary_header(dtb_ipa);
    crate::uart_println!("\t\tDTB first bytes");
    for i in 0..4 { crate::uart_println!("\t\t{:02x}", GUEST1_DTB[i]); }
    crate::uart_println!("\t\tDTB SIZE = {}", GUEST1_DTB_SIZE);

    // 3) copy initramfs to guest memory if any
    if let Some(initramfs) = cfg.image_initramfs {
        let initramfs_ipa = GUEST1_IPA_BASE + 0x0300_0000;

        crate::uart_println!("\tCopying initramfs to guest memory...");
        copy_blob_to_guest(
            GUEST1_PA_BASE,
            GUEST1_IPA_BASE,
            initramfs_ipa,
            initramfs
        );
        
        crate::uart_println!("\t| CHECK | no overlap");
        let initramfs_start = initramfs_ipa;
        let initramfs_end   = initramfs_ipa + initramfs.len() as u64 - 1;
        crate::uart_println!(
            "\t\tInitramfs IPA: [0x{:016x} - 0x{:016x}]",
            initramfs_start, initramfs_end
        );

        crate::uart_println!("\t| CHECK | initramfs dump");
        dump_binary_header(initramfs_ipa);
        crate::uart_println!("\t\tINITRAMFS DATA first bytes");
        for i in 0..4 { crate::uart_println!("\t\t{:02x}", GUEST1_INITRAMFS_DATA[i]); }
        crate::uart_println!("\t\tINITRAMFS SIZE = {}", GUEST1_INITRAMFS_SIZE);
    }

    // 4) Stage‑2 mapping for linux guest
    // RAM
    for off in (0..GUEST1_IPA_SIZE).step_by(4096) {
        let ipa = GUEST1_IPA_BASE + off;
        ArchVm::map_page(
            ipa,
            GUEST1_PA_BASE  + off
        );
    }
    crate::uart_println!("\tLinux guest RAM mapped: IPA 0x{:016x} - 0x{:016x} -> PA 0x{:016x} - 0x{:016x}",
        GUEST1_IPA_BASE, GUEST1_IPA_BASE + GUEST1_IPA_SIZE - 1,
        GUEST1_PA_BASE, GUEST1_PA_BASE + GUEST1_IPA_SIZE - 1
    );

    crate::uart_println!("\tMapping devices...");
    // GICv2
    ArchVm::map_device(GUEST_GICD_IPA, GUEST_GICD_IPA); // GICD
    crate::uart_println!("\t\tGICD mapped");
    ArchVm::map_device(GUEST_GICC_IPA, GUEST_GICC_IPA); // GICC
    crate::uart_println!("\t\tGICC mapped");

    // UART
    ArchVm::map_device(GUEST_UART_IPA, GUEST_UART_IPA); // UART
    crate::uart_println!("\t\tUART mapped");

    crate::uart_println!("\t\tDevice mapping done.");

    // 5) Register the VM
    VMS[cfg.id as usize] = Some(VM {
        id: cfg.id as usize,
        ipa_base: cfg.ipa_range.0,
        ipa_size: cfg.ipa_range.1 - cfg.ipa_range.0 + 1,
        entry: LINUX_ENTRY_IPA,
        s2_root: ArchVm::get_s2_root(),
        //dtb_ipa: Some(dtb_ipa),
    });
    crate::uart_println!("\tLinux guest registered in VMS table: id = {:02x}, ipa_base = 0x{:016x}, ipa_size = 0x{:016x}, entry = 0x{:016x}, s2_root = 0x{:016x}",
        cfg.id, cfg.ipa_range.0, cfg.ipa_range.1 - cfg.ipa_range.0 + 1, LINUX_ENTRY_IPA, ArchVm::get_s2_root()
    );

    return dtb_ipa;
}

pub unsafe fn run_linux_vm(id: usize, dtb_ipa: u64) -> ! {
    crate::uart_println!("Preparing to run Linux VM...");
    
    // unwrap VM from VMS table
    crate::uart_println!("\tUnwrap VM...");
    let vm = VMS[id].unwrap();
    crate::uart_println!("\t\tVM entry = 0x{:016x}", vm.entry);

    // IPA -> PA pour l’entrée
    let entry_pa = GUEST1_PA_BASE + (LINUX_ENTRY_IPA - GUEST1_IPA_BASE);
    // IPA -> PA pour le DTB
    let dtb_pa   = GUEST1_PA_BASE + (dtb_ipa        - GUEST1_IPA_BASE);

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
        (0b0101 << 0) |   // EL1h
        (1 << 6)      |   // F
        (1 << 7)      |   // I
        (1 << 8);         // A

    crate::uart_println!("\t\tspsr     = 0x{:016x}", spsr);

    crate::uart_println!("\t|CHECK| kernel @PA 0x{:016x}", entry_pa);

    // Launch VM...
    crate::uart_println!("\n---------- Running VM... ----------\n");

    core::arch::asm!(
        "mov x0, {dtb}",        // x0 = dtb PA
        "mov x1, xzr",
        "mov x2, xzr",
        "mov x3, xzr",
        "msr elr_el2, {entry}", // PC = PA
        "msr spsr_el2, {psr}",
        "eret",
        dtb   = in(reg) dtb_ipa,//dtb_pa,
        entry = in(reg) LINUX_ENTRY_IPA,//entry_pa
        psr   = in(reg) spsr,
    );

    loop {}
}

