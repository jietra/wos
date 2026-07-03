// src/arch/aarch64/mmu/mod.rs

pub mod tables;

pub use tables::{init_page_tables, L0_TABLE};

use crate::arch::aarch64::boot::linker_symbols as ls;
use crate::memory::memory_layout::layout::KERNEL_BASE;

// -----------------------------------------------------------------------------
// MAIR initialization (sets up memory attributes for normal and device memory)
// -----------------------------------------------------------------------------
pub unsafe fn init_mair() {
    let mair_value: u64 =
        (0x00 << 0)  |   // Attr0 = Device-nGnRnE
        (0x44 << 8)  |   // Attr1 = Normal Non-Cacheable
        (0xFF << 16);    // Attr2 = Normal Write-Back Cacheable

    core::arch::asm!(
        "msr MAIR_EL1, {0}",
        in(reg) mair_value,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// TCR initialization (sets up translation control for 48-bit VA space, 4KB granules, etc.)
// -----------------------------------------------------------------------------
pub unsafe fn init_tcr() {
    let tcr_value: u64 =
        // TTBR0 (low VA)
        (16 << 0) |        // T0SZ = 16 -> 48-bit VA space
        (0b00 << 14) |     // TG0 = 4KB granule
        (0b11 << 12) |     // SH0 = Inner Shareable
        (0b01 << 10) |     // ORGN0 = Write-back cacheable
        (0b01 << 8)  |     // IRGN0 = Write-back cacheable

        // TTBR1 (high VA)
        (16 << 16) |       // T1SZ = 48-bit VA
        (0b10 << 30) |     // TG1 = 4KB (note: encoding different from TG0)
        (0b11 << 28) |     // SH1 = Inner Shareable
        (0b01 << 26) |     // ORGN1 = WB
        (0b01 << 24) |     // IRGN1 = WB

        // physical address size
        (0b101 << 32);     // IPS = 48-bit PA 

    core::arch::asm!(
        "msr TCR_EL1, {0}",
        in(reg) tcr_value,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// TTBR0-TTBR1 initialization (set to point to L0 page table)
// -----------------------------------------------------------------------------
pub unsafe fn init_ttbr0() {
    let l0_addr = &raw const L0_TABLE as *const _ as u64;
    core::arch::asm!(
        "msr TTBR0_EL1, {0}",
        in(reg) l0_addr,
        options(nostack, preserves_flags),
    );
}

pub unsafe fn init_ttbr1() {
    let l0_addr = &raw const L0_TABLE as *const _ as u64;
    core::arch::asm!(
        "msr TTBR1_EL1, {0}",
        in(reg) l0_addr,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// Enable MMU (sets SCTLR_EL1 to enable MMU, caches, etc.)
// -----------------------------------------------------------------------------
pub unsafe fn enable_mmu() {
    let mut sctlr: u64;

    core::arch::asm!(
        "mrs {0}, SCTLR_EL1",
        out(reg) sctlr,
        options(nostack, preserves_flags),
    );

    // Enable MMU (M), data cache (C), and instruction cache (I)
    sctlr |= 1 << 0;   // M
    sctlr |= 1 << 2;   // C
    sctlr |= 1 << 12;  // I

    core::arch::asm!(
        "msr SCTLR_EL1, {0}",
        "isb",
        in(reg) sctlr,
        options(nostack, preserves_flags),
    );

    // switch uart address to va (otherwise no prints)
    crate::arch::mmio::UART_BASE = crate::arch::mmio::UART_VADDR;

    crate::uart_println!("\tMMU enabled.");
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
// Set VBAR in VA
// -----------------------------------------------------------------------------

extern "C" {
    static _exceptions_start: u8;
}

pub unsafe fn set_vbar_in_va() {
    let mut vbar: u64;
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1 before set vbar in va @ 0x", vbar);

    let exc_phys = &_exceptions_start as *const u8 as u64;
    let exc_virt = tables::phys_to_kernel_virt(exc_phys); // same logic as for .text

    core::arch::asm!(
        "msr VBAR_EL1, {0}",
        in(reg) exc_virt,
        options(nostack, preserves_flags),
    );

    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\tVBAR_EL1 after  set vbar in va @ 0x", vbar);

    crate::uart_println!("\t\texceptions phys address = ", exc_phys);
    crate::uart_println!("\t\texceptions virt address = ", exc_virt);
}