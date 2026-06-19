// arch/aarch64/mmu/mod.rs

pub mod tables;

pub use tables::init_page_tables;
pub use tables::L0_TABLE;

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
        (16 << 0) |        // T0SZ = 16 → 48-bit VA space
        (0b00 << 14) |     // TG0 = 4KB granule
        (0b11 << 12) |     // SH0 = Inner Shareable
        (0b01 << 10) |     // ORGN0 = Write-back cacheable
        (0b01 << 8)  |     // IRGN0 = Write-back cacheable
        (0b101 << 32);     // IPS = 48-bit physical address size

    core::arch::asm!(
        "msr TCR_EL1, {0}",
        in(reg) tcr_value,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// TTBR0 initialization (sets TTBR0_EL1 to point to our L0 page table)
// -----------------------------------------------------------------------------
pub unsafe fn init_ttbr0() {
    let l0_addr = &raw const L0_TABLE as *const _ as u64;

    core::arch::asm!(
        "msr TTBR0_EL1, {0}",
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
}