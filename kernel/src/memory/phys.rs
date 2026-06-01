// -----------------------------------------------------------------------------
// Simple physical page allocator (bump allocator)
// -----------------------------------------------------------------------------
static mut NEXT_FREE_PHYS: u64 = 0;
static mut PHYS_LIMIT: u64 = 0;

pub unsafe fn init_phys_alloc(kernel_end: u64) {
    let aligned = (kernel_end + 0xFFF) & !0xFFF;
    NEXT_FREE_PHYS = aligned;
    PHYS_LIMIT = 0x8000_0000;
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