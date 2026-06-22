// -----------------------------------------------------------------------------
// Virtual memory management
// -----------------------------------------------------------------------------
// For future use: This module will contain functions for managing virtual memory, including page table management, address translation, etc. For now, it just contains some helper functions for working with virtual addresses and page tables.

/*
pub fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

pub fn l0_index(va: u64) -> usize {
    ((va >> 39) & 0x1FF) as usize
}

pub fn l1_index(va: u64) -> usize {
    ((va >> 30) & 0x1FF) as usize
}

pub fn l2_index(va: u64) -> usize {
    ((va >> 21) & 0x1FF) as usize
}

pub fn l3_index(va: u64) -> usize {
    ((va >> 12) & 0x1FF) as usize
}
*/
