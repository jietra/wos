// src/arch/aarch64/mmu/mod.rs

pub mod tables;
pub mod stage2;

pub use tables::{init_page_tables, L0_TABLE, L0_LOW, L0_HIGH};

use crate::arch::aarch64::boot::linker_symbols as ls;
use crate::memory::memory_layout::layout::KERNEL_BASE;

// -----------------------------------------------------------------------------
// MAIR initialization (sets up memory attributes for normal and device memory)
// -----------------------------------------------------------------------------
pub unsafe fn init_mair() {
    crate::uart_println!("\tInitializing Memory Attributes for MAIR_EL1...");
    /*let mair_value: u64 =
        (0x00 << 0)  |   // Attr0 = Device-nGnRnE
        (0x44 << 8)  |   // Attr1 = Normal Non-Cacheable
        (0xFF << 16);    // Attr2 = Normal Write-Back Cacheable
    */
    let mut mair_value: u64 = (0x00 << 0);   // Attr0 = Device-nGnRnE
    crate::uart_println!("\t\tMAIR_EL1 Attr0 (0x00 << 0) Device-nGnRnE                = {}", mair_value);
    mair_value |= (0x44 << 8);   // Attr1 = Normal Non-Cacheable
    crate::uart_println!("\t\tMAIR_EL1 Attr1 (0x44 << 8) Normal Non-Cacheable         = {}", mair_value);
    mair_value |= (0xFF << 16);  // Attr2 = Normal Write-Back Cacheable
    crate::uart_println!("\t\tMAIR_EL1 Attr2 (0xFF << 16) Normal Write-Back Cacheable = {}", mair_value);

    core::arch::asm!(
        "msr MAIR_EL1, {0}",
        in(reg) mair_value,
        options(nostack, preserves_flags),
    );

    // log value to check
    let mut mair: u64;
    core::arch::asm!("mrs {0}, MAIR_EL1", out(reg) mair);
    crate::uart_println!("\t\tMAIR_EL1 (0x0000000000ff4400 expected)        = 0x{:016x}", mair);
}

// -----------------------------------------------------------------------------
// TCR initialization (sets up translation control for 48-bit VA space, 4KB granules, etc.)
// -----------------------------------------------------------------------------
pub unsafe fn init_tcr() {
    crate::uart_println!("\tInitializing Translation Control for TCR_EL1...");

    /*let tcr_value: u64 =
        // TTBR0 (low VA)
        (16    << 0 ) |     // T0SZ  = 16 -> 48-bit VA space
        (0b00  << 14) |     // TG0   = 4KB
        (0b11  << 12) |     // SH0   = Inner Shareable
        (0b01  << 10) |     // ORGN0 = Write-back cacheable
        (0b01  << 8 ) |     // IRGN0 = Write-back cacheable
        // TTBR1 (high VA)
        (16    << 16) |     // T1SZ = 48-bit VA
        (0b10  << 30) |     // TG1 = 4KB (note: encoding different from TG0)
        (0b11  << 28) |     // SH1 = Inner Shareable
        (0b01  << 26) |     // ORGN1 = WB
        (0b01  << 24) |     // IRGN1 = WB
        // physical address size
        (0b101 << 32);      // IPS   = 48-bit PA
    */

    // TTBR0 (low VA)
    let t0sz : u64 = (16   << 0 );              // T0SZ  = 16 -> 48-bit VA space
    let tg0  : u64 = (0b00 << 14);              // TG0   = 4KB
    let sh0  : u64 = (0b11 << 12);              // SH0   = Inner Shareable
    let orgn0: u64 = (0b01 << 10);              // ORGN0 = Write-back cacheable
    let irgn0: u64 = (0b01 << 8 );              // IRGN0 = Write-back cacheable
    // TTBR1 (high VA)
    let t1sz : u64 = (16   << 16);              // T1SZ = 48-bit VA
    let tg1  : u64 = (0b00 << 30); // 4 KiB vs (0b10 << 30);              // TG1 = 4KB (note: encoding different from TG0)
    let sh1  : u64 = (0b11 << 28);              // SH1 = Inner Shareable
    let orgn1: u64 = (0b01 << 26);              // ORGN1 = WB
    let irgn1: u64 = (0b01 << 24);              // IRGN1 = WB
    // physical address size
    let ips  : u64 = ((0b101 as u64) << 32);    // IPS   = 48-bit PA

    crate::uart_println!("\t\t--- TTBR0 (low VA) ---");
    crate::uart_println!("\t\tT0SZ  (16    << 0 )   = 0x{:016x}", t0sz);    
    crate::uart_println!("\t\tTG0   (0b00  << 14)   = 0x{:016x}", tg0);     
    crate::uart_println!("\t\tSH0   (0b11  << 12)   = 0x{:016x}", sh0);     
    crate::uart_println!("\t\tORGN0 (0b01  << 10)   = 0x{:016x}", orgn0);   
    crate::uart_println!("\t\tIRGN0 (0b01  << 8 )   = 0x{:016x}", irgn0);
    crate::uart_println!("\t\t--- TTBR1 (high VA) ---");
    crate::uart_println!("\t\tT1SZ  (16    << 16)   = 0x{:016x}", t1sz);    
    crate::uart_println!("\t\tTG1   (0b00  << 30)   = 0x{:016x}", tg1);     
    crate::uart_println!("\t\tSH1   (0b11  << 28)   = 0x{:016x}", sh1);     
    crate::uart_println!("\t\tORGN1 (0b01  << 26)   = 0x{:016x}", orgn1);   
    crate::uart_println!("\t\tIRGN1 (0b01  << 24)   = 0x{:016x}", irgn1);
    crate::uart_println!("\t\t--- Phys addr  size ---");
    crate::uart_println!("\t\tIPS   (0b101 << 32)   = 0x{:016x}", ips);

    crate::uart_println!("\t\t--- TCR_EL1 comput° ---");
    crate::uart_println!("\t\tTCR_EL1 = T0SZ                        = 0x{:016x}", t0sz);
    crate::uart_println!("\t\tTCR_EL1 = T0SZ|TG0                    = 0x{:016x}", (t0sz | tg0));
    crate::uart_println!("\t\tTCR_EL1 = T0SZ|TG0|SH0                = 0x{:016x}", (t0sz | tg0 | sh0));
    crate::uart_println!("\t\tTCR_EL1 = T0SZ|TG0|SH0|ORGN0          = 0x{:016x}", (t0sz | tg0 | sh0 | orgn0));   
    crate::uart_println!("\t\tTCR_EL1 = T0SZ|TG0|SH0|ORGN0|IRGN0    = 0x{:016x}", (t0sz | tg0 | sh0 | orgn0|irgn0));
    crate::uart_println!("\t\tTCR_EL1 = T0SZ|TG0|SH0|ORGN0|IRGN0|IPS= 0x{:016x}", (t0sz | tg0 | sh0 | orgn0|irgn0|ips));

    // TCR value
    //let tcr_value: u64 = t0sz | tg0 | sh0 | orgn0 | irgn0 | ips;
    let tcr_value: u64 = t0sz | tg0 | sh0 | orgn0 | irgn0 | t1sz | tg1 | sh1 | orgn1 | irgn1 | ips;
    core::arch::asm!(
        "msr TCR_EL1, {0}",
        "isb",
        in(reg) tcr_value,
        options(nostack, preserves_flags)
    );

    // log value to check
    let mut tcr: u64;
    core::arch::asm!("mrs {0}, TCR_EL1", out(reg) tcr);
    crate::uart_println!("\t\tTCR_EL1 = 0x{:016x}", tcr);
}

// -----------------------------------------------------------------------------
// TTBR0-TTBR1 initialization (set to point to L0 page table)
// -----------------------------------------------------------------------------
pub unsafe fn init_ttbr0() {
    crate::uart_println!("\tInitializing Translation Table Base Register 0 (TTBR0_EL1)...");
    let l0_va = &L0_LOW as *const _ as u64;
    let k_va = &ls::_kernel_start as *const u8 as u64;
    let k_pa = crate::arch::aarch64::mmu::tables::kernel_start_phys();
    let l0_pa = k_pa + (l0_va - k_va);
    crate::uart_println!("\t\tL0_LOW VA = 0x{:016x}", l0_va);
    crate::uart_println!("\t\tKernel VA = 0x{:016x}", k_va);
    crate::uart_println!("\t\tKernel PA = 0x{:016x}", k_pa);
    crate::uart_println!("\t\tL0_LOW PA = 0x{:016x}", l0_pa);

    core::arch::asm!(
        "msr TTBR0_EL1, {0}",
        in(reg) l0_pa,
        options(nostack, preserves_flags),
    );

    // log value to check
    let mut ttbr0: u64;
    core::arch::asm!("mrs {0}, TTBR0_EL1", out(reg) ttbr0);
    crate::uart_println!("\t\tTTBR0_EL1 (0x401E1000 expected) = 0x{:016x}", ttbr0);

    crate::uart_println!("\n\t\t_kernel_start      = 0x{:016x}", &ls::_kernel_start as *const u8 as u64);
    crate::uart_println!(  "\t\t_stack_top         = 0x{:016x}", &ls::_stack_top    as *const u8 as u64);
    crate::uart_println!(  "\t\t_boot_tables_start = 0x{:016x}", &ls::_boot_tables_start as *const u8 as u64);
    crate::uart_println!(  "\t\t_boot_tables_end   = 0x{:016x}", &ls::_boot_tables_end   as *const u8 as u64);
    crate::uart_println!(  "\t\tL0_LOW VA          = 0x{:016x}", &L0_LOW as *const _ as u64);

}

pub unsafe fn init_ttbr1() {
    crate::uart_println!("\tInitializing Translation Table Base Register 1 (TTBR1_EL1)...");
    let l0_va = &L0_HIGH as *const _ as u64;
    let k_va = &ls::_kernel_start as *const u8 as u64;
    let k_pa = crate::arch::aarch64::mmu::tables::kernel_start_phys();

    let l0_pa = k_pa + (l0_va - k_va);
    core::arch::asm!(
        "msr TTBR1_EL1, {0}",
        in(reg) l0_pa,
        options(nostack, preserves_flags),
    );

    let mut ttbr1: u64;
    core::arch::asm!("mrs {0}, TTBR1_EL1", out(reg) ttbr1);
    crate::uart_println!("\t\tTTBR1_EL1 = 0x{:016x}", ttbr1);
}

// -----------------------------------------------------------------------------
// Enable MMU (sets SCTLR_EL1 to enable MMU, caches, etc.)
// -----------------------------------------------------------------------------
pub unsafe fn enable_mmu() {
    crate::uart_println!("\tenable MMU...");

    /*
    // CHECK VBAR (Debug purpose)
    crate::uart_println!("\t\t--- checking VBAR_EL1 ---");
    let mut vbar: u64;
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1             = 0x{:016x}", vbar);
    let exc_va = &ls::_exceptions_start as *const u8 as u64;
    core::arch::asm!("msr VBAR_EL1, {0}", in(reg) exc_va);
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1 (after set) = 0x{:016x}", vbar);
    */

    // CHECK translation tables for a test VA
    crate::uart_println!("\t\t---checking translation tables for a test VA ---");
    let ks = 0x0000000040080000u64; // boot region start
    let va = ks;                    // a VA in the identity zone
    let i0 = crate::memory::virt::l0_index(va);
    let i1 = crate::memory::virt::l1_index(va);
    let i2 = crate::memory::virt::l2_index(va);
    let i3 = crate::memory::virt::l3_index(va);

    crate::uart_println!("\t\ttest_va = 0x{:016x}", va);
    crate::uart_println!("\t\tindices: L0={}", i0);
    crate::uart_println!("\t\tindices: L1={}", i1);
    crate::uart_println!("\t\tindices: L2={}", i2);
    crate::uart_println!("\t\tindices: L3={}", i3);
    crate::uart_println!("\t\tL0_LOW[i0] = 0x{:016x}", crate::arch::aarch64::mmu::tables::L0_LOW.0[i0]);

    let l1_pa = crate::arch::aarch64::mmu::tables::L0_LOW.0[i0] & !0xFFF;
    crate::uart_println!("\t\tL1_PA      = 0x{:016x}", l1_pa);

    let l1 = l1_pa as *const u64;
    crate::uart_println!("\t\tL1[i1]     = 0x{:016x}", unsafe { *l1.add(i1) });

    let l2_pa = unsafe { *l1.add(i1) } & !0xFFF;
    crate::uart_println!("\t\tL2_PA      = 0x{:016x}", l2_pa);

    let l2 = l2_pa as *const u64;
    crate::uart_println!("\t\tL2[i2]     = 0x{:016x}", unsafe { *l2.add(i2) });

    let l3_pa = unsafe { *l2.add(i2) } & !0xFFF;
    crate::uart_println!("\t\tL3_PA      = 0x{:016x}", l3_pa);

    let l3 = l3_pa as *const u64;
    crate::uart_println!("\t\tL3[i3]     = 0x{:016x}", unsafe { *l3.add(i3) });

    crate::uart_println!("\t\tVA 0x{:016x} -> PA 0x{:016x}", va, unsafe { *l3.add(i3) } & !0xFFF);

    // Reading SCTLR_EL1 before enabling MMU
    crate::uart_println!("\t\treading SCTLR...");
    let mut sctlr: u64;
    core::arch::asm!(
        "mrs {0}, SCTLR_EL1",
        out(reg) sctlr,
        options(nostack, preserves_flags),
    );
    crate::uart_println!("\t\tSCTLR_EL1 before MMU  = 0x{:016x}", sctlr);

    // Enable MMU (M), data cache (C), and instruction cache (I)
    sctlr |= 1 << 0;   // M
    crate::uart_println!("\t\tSCTLR_EL1 after M     = 0x{:016x}", sctlr);

    sctlr |= 1 << 2;   // C
    crate::uart_println!("\t\tSCTLR_EL1 after M/C   = 0x{:016x}", sctlr);

    sctlr |= 1 << 12;  // I
    crate::uart_println!("\t\tSCTLR_EL1 after M/C/I = 0x{:016x}", sctlr);

    crate::uart_println!("\t\twriting SCTLR...");

    core::arch::asm!(
        "msr SCTLR_EL1, {0}",
        "isb",
        in(reg) sctlr,
        options(nostack, preserves_flags),
    );

    // CHECK call enable_mmu in high half (VA) after enabling MMU
    crate::uart_println!("\t\t--- checking enable_mmu address ---");
    let f_phys = enable_mmu as *const () as u64;
    crate::uart_println!("\t\tenable_mmu phys @ 0x{:016x}", f_phys);
    let f_virt = KERNEL_BASE as u64 + (f_phys - crate::arch::aarch64::mmu::tables::kernel_start_phys());
    crate::uart_println!("\t\tenable_mmu virt = 0x{:016x}", f_virt);

    let va = f_virt;
    let i0 = crate::memory::virt::l0_index(va);
    let i1 = crate::memory::virt::l1_index(va);
    let i2 = crate::memory::virt::l2_index(va);
    let i3 = crate::memory::virt::l3_index(va);

    crate::uart_println!("\t\tf_virt VA = 0x{:016x}", va);
    crate::uart_println!("\t\tindices: L0={}", i0);
    crate::uart_println!("\t\tindices: L1={}", i1);
    crate::uart_println!("\t\tindices: L2={}", i2);
    crate::uart_println!("\t\tindices: L3={}", i3);
    crate::uart_println!("\t\tL0_HIGH[i0] = 0x{:016x}", L0_HIGH.0[i0]);

    let l1_pa = L0_HIGH.0[i0] & !0xFFF;
    let l1 = l1_pa as *const u64;
    crate::uart_println!("\t\tL1[i1] = 0x{:016x}", unsafe { *l1.add(i1) });

    let l2_pa = unsafe { *l1.add(i1) } & !0xFFF;
    let l2 = l2_pa as *const u64;
    crate::uart_println!("\t\tL2[i2] = 0x{:016x}", unsafe { *l2.add(i2) });

    let l3_pa = unsafe { *l2.add(i2) } & !0xFFF;
    let l3 = l3_pa as *const u64;
    crate::uart_println!("\t\tL3[i3] = 0x{:016x}", unsafe { *l3.add(i3) });

    let pa = unsafe { *l3.add(i3) & !0xFFF };
    crate::uart_println!("\t\tf_virt VA -> PA = 0x{:016x}", pa);

    crate::uart_println!("\t\t--- end check ---");

    // switch uart address to va (otherwise no prints)
    crate::uart_println!("\t\tSwitching UART address to virtual address...");
    crate::arch::mmio::UART_BASE = crate::arch::mmio::UART_VADDR;

    /*
    // CHECK still alive (Debug purpose)
    let mut c: u64 = 0;
    loop {
        core::arch::asm!("nop");
        c += 1;
        if c % 100_000_000 == 0 {
            crate::uart_println!("still alive, c={}", c);
        }
    }
    */

    crate::uart_println!("\t\tMMU enabled.");
}






/*
/// For debug: used during switch for phys addresses to virt addresses (MMU activation)
#[inline(always)]
pub fn mmu_is_enabled() -> bool {
    let sctlr: u64;
    unsafe {
        core::arch::asm!("mrs {0}, SCTLR_EL1", out(reg) sctlr);
    }
    (sctlr & 1) != 0
}*/

// -----------------------------------------------------------------------------
// Set VBAR in VA [TODO: already done in enable_mmu(): can be removed]
// -----------------------------------------------------------------------------

extern "C" {
    static _exceptions_start: u8;
}

pub unsafe fn set_vbar_in_va() {
    let mut vbar: u64;
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1 before set vbar in va @ 0x{}", vbar);

    let exc_phys = &_exceptions_start as *const u8 as u64;
    let exc_virt = phys_to_kernel_virt(exc_phys); // same logic as for .text

    core::arch::asm!(
        "msr VBAR_EL1, {0}",
        in(reg) exc_virt,
        options(nostack, preserves_flags),
    );

    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1 after  set vbar in va @ 0x{}", vbar);

    crate::uart_println!("\t\texceptions phys address = {}", exc_phys);
    crate::uart_println!("\t\texceptions virt address = {}", exc_virt);
}

pub fn phys_to_kernel_virt(pa: u64) -> u64 {
    let ks = crate::arch::aarch64::mmu::tables::kernel_start_phys();
    // temporary: avoid underflow
    if pa < ks {
        // log / println / return pa (debug)
        return pa;
    }
    KERNEL_BASE as u64 + (pa - ks)
}