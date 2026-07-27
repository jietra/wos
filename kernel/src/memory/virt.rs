// src/memory/virt.rs

// -----------------------------------------------------------------------------
// Virtual memory management
// -----------------------------------------------------------------------------

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
