use crate::arch::aarch64::linker_symbols::_kernel_start;
use crate::arch::aarch64::linker_symbols::_kernel_end;

// -----------------------------------------------------------------------------
// Minimal page tables (4-level, 512 entries each, 4KB pages)
// -----------------------------------------------------------------------------
#[repr(align(4096))]
pub struct PageTable(pub [u64; 512]);

pub static mut L0_TABLE: PageTable = PageTable([0; 512]);
pub static mut L1_TABLE: PageTable = PageTable([0; 512]);
pub static mut L2_KERNEL_TABLE: PageTable = PageTable([0; 512]);

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

// Helper function to create a block entry in the L2 page table (maps 2 MB blocks)
pub fn l2_block_entry(phys: u64, attr_index: u64, executable: bool) -> u64 {
    let mut desc =
        (phys & !((1u64 << 21) - 1)) | // align 2 Mo
        (attr_index << 2) |
        (1 << 10) |
        (3 << 8)  |
        (0 << 6)  |
        0b01; // VALID + BLOCK (L2)

    if !executable {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}

// -----------------------------------------------------------------------------
// Minimal page tables initialization (maps 0x4008_0000 for kernel and 0x0900_0000 for UART)
// -----------------------------------------------------------------------------
pub unsafe fn init_page_tables() {
    // L0[0] -> L1
    L0_TABLE.0[0] = (&raw const L1_TABLE as *const _ as u64) | 0b11;

    // L1[0]: 0–1 Go: identity mapping of the first 1 Go of physical memory (where the kernel and peripherals are located) in Normal WB, RW, executable
    let phys0 = 0u64 << 30;
    L1_TABLE.0[0] = l1_block_entry(phys0, 2, true);

    // L1[1]: 1–2 Go -> table L2_KERNEL
    L1_TABLE.0[1] = (&raw const L2_KERNEL_TABLE as *const _ as u64) | 0b11;

    // We fill L2_KERNEL_TABLE with 2 Mo blocks mapping the kernel region (0x4008_0000–0x7FFF_FFFF) in Normal WB, RW, executable
    // 1-2 Go region base physical address (where the kernel is loaded by the bootloader, defined in the linker script) – we assume the kernel is loaded at 1 Go (0x4000_0000) for simplicity, but in a real kernel you'd want to use the actual load address defined in your linker script
    let l1_1_base_phys = 1u64 << 30; // 0x4000_0000
    // Start/End kernel virtual addresses (identical to physical since on identity mapping + Virtual Address = Physical Address for 1–2 Go) defined in the linker script
    let kernel_start = &_kernel_start as *const u8 as u64;
    let kernel_end   = &_kernel_end   as *const u8 as u64;
    // We compute the first and last L2 indices that cover the kernel region
    // L1[1] covers 0x4000_0000–0x7FFF_FFFF
    // L2 index = bits [29:21]
    let first_l2 = (((kernel_start - l1_1_base_phys) >> 21) & 0x1FF) as usize;
    let last_l2  = (((kernel_end   - l1_1_base_phys) >> 21) & 0x1FF) as usize;
    for i in first_l2..=last_l2 {
        let phys = l1_1_base_phys + ((i as u64) << 21); // 2 Mo
        // For now: we mark the whole 1–2 Go region as executable, but in a real kernel you'd want to mark only the actual code sections as executable and the rest (data, bss, peripherals) as non-executable
        L2_KERNEL_TABLE.0[i] = l2_block_entry(phys, 2, true);
    }
}