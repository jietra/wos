use crate::arch::aarch64::linker_symbols as ls;

// -----------------------------------------------------------------------------
// Minimal page tables
// (4-level, 512 entries each, 4KB pages)
// -----------------------------------------------------------------------------
#[repr(align(4096))]
pub struct PageTable(pub [u64; 512]);

pub static mut L0_TABLE: PageTable = PageTable([0; 512]);
pub static mut L1_TABLE: PageTable = PageTable([0; 512]);
pub static mut L2_TABLE: PageTable = PageTable([0; 512]);
pub static mut L3_KERNEL_TABLE: PageTable = PageTable([0; 512]);

// Helper function to create a block entry in the page table
pub fn l1_block_entry(phys: u64, attr: u64, exec: bool) -> u64 {
    let mut desc = (phys & !((1 << 30) - 1))    // Align physical address to 1 GB // use 0x0000_FFFF_FFFF_0000 or !((1<<12)-1) for 4KB pages alignment (L3), !((1<<21)-1) for 2MB blocks (L2)
        | (attr << 2)
        | (1 << 10)                             // AF
        | (3 << 8)                              // SH = Inner Shareable
        | (0 << 6)                              // AP = RW EL1
        | 0b01;                                 // VALID + BLOCK

    if !exec {
        // PXN + UXN = Execute Never for both privileged and unprivileged
        desc |= (1 << 54) | (1 << 53);
    }

    desc
}

// Helper function
pub fn l3_page_entry(phys: u64, attr_index: u64, executable: bool, ap: u64) -> u64 {
    let mut desc =
        (phys & !((1u64 << 12) - 1)) |      // align 4 KiB
        (attr_index << 2) |
        (1 << 10) |                         // AF
        (3 << 8)  |                         // SH = Inner Shareable
        (ap << 6)  |                        // AP bits
        0b11;                               // VALID + PAGE

    if !executable {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}

// Helper functions to map .text
pub unsafe fn map_kernel_l3() {
    let l1_1_base = 1u64 << 30;     // 0x4000_0000

    let text_start   = &ls::_text_start   as *const u8 as u64;
    let text_end     = &ls::_text_end     as *const u8 as u64;
    let rodata_start = &ls::_rodata_start as *const u8 as u64;
    let rodata_end   = &ls::_rodata_end   as *const u8 as u64;
    let data_start   = &ls::_data_start   as *const u8 as u64;
    let data_end     = &ls::_data_end     as *const u8 as u64;
    let bss_start    = &ls::_bss_start    as *const u8 as u64;
    let bss_end      = &ls::_bss_end      as *const u8 as u64;

    // Suppose all fits in the same 2 MiB bloc
    let l2_index = (((text_start - l1_1_base) >> 21) & 0x1FF) as usize;
    let l2_region_base = l1_1_base + ((l2_index as u64) << 21);

    // L2 -> L3
    L2_TABLE.0[l2_index] = (&raw const L3_KERNEL_TABLE as *const _ as u64) | 0b11;

    // Fills the 512 4K pages of the 2 MiB bloc
    for i in 0..512 {
        let va = l2_region_base + ((i as u64) << 12);
        let phys = va;              // identity mapping

        let (attr_index, ap, exec) = if va >= text_start && va < text_end {
            (2, 0b10, true)         // .text : Normal WB, RO, executable
        } else if va >= rodata_start && va < rodata_end {
            (2, 0b10, false)        // .rodata : Normal WB, RO, XN
        } else if (va >= data_start && va < data_end) || (va >= bss_start && va < bss_end) {
            (2, 0b00, false)        // .data/.bss : Normal WB, RW, XN
        } else {
            (2, 0b00, false)        // the rest: RW, XN
        };

        L3_KERNEL_TABLE.0[i] = l3_page_entry(phys, attr_index, exec, ap);
    }
}

// -----------------------------------------------------------------------------
// Minimal page tables initialization
// (maps 0x4008_0000 for kernel and 0x0900_0000 for UART)
// -----------------------------------------------------------------------------
pub unsafe fn init_page_tables() {
    L0_TABLE.0[0] = (&raw const L1_TABLE as *const _ as u64) | 0b11;    // L0[0] -> L1

    L1_TABLE.0[0] = l1_block_entry(0, 0, false);                        // L1[0]: 0–1 Go: Device
    L1_TABLE.0[1] = (&raw const L2_TABLE as *const _ as u64) | 0b11;    // L1[1]: 1–2 Go -> KERNEL

    map_kernel_l3();
}