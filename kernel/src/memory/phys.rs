// src/memory/phys.rs

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::boot::linker_symbols as ls;

// -----------------------------------------------------------------------------
// Simple physical page allocator (bump allocator)
// -----------------------------------------------------------------------------
pub static mut PAGE_POOL_START: u64 = 0;
pub static mut PAGE_POOL_END: u64 = 0;
static mut NEXT_TABLE: u64 = 0;

static mut NEXT_FREE_PHYS: u64 = 0;
pub static mut PHYS_LIMIT: u64 = 0;

#[cfg(target_arch = "aarch64")]
pub unsafe fn init_page_pool() {
    PAGE_POOL_START = &ls::_page_pool_start as *const u8 as u64;
    PAGE_POOL_END   = &ls::_page_pool_end   as *const u8 as u64;
    NEXT_TABLE      = PAGE_POOL_START;
}

pub unsafe fn init_phys_alloc(kernel_end: u64) {
    crate::uart_println!("\tInitializing physical memory allocator...");

    let aligned = (kernel_end + 0xFFF) & !0xFFF;
    NEXT_FREE_PHYS = aligned;
    PHYS_LIMIT = 0x6300_0000; //0x8000_0000;
}

pub unsafe fn alloc_page() -> Option<u64> {
    if NEXT_FREE_PHYS + 0x1000 > PHYS_LIMIT {
        None
    } else {
        let p = NEXT_FREE_PHYS;
        NEXT_FREE_PHYS += 0x1000;
        Some(p)
    }
}

// -----------------------------------------------------------------------------
// MMU page table allocation
// -----------------------------------------------------------------------------
pub unsafe fn alloc_page_table() -> Option<*mut u64> {
    //alloc_page().map(|pa| pa as *mut u64)
    if NEXT_TABLE + 0x1000 > PAGE_POOL_END {
        None
    } else {
        let p = NEXT_TABLE;
        NEXT_TABLE += 0x1000;
        Some(p as *mut u64)
    }
}