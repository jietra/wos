// -----------------------------------------------------------------------------
// MMU page table allocation
// -----------------------------------------------------------------------------
// For future expansion, we can add support for freeing pages, keeping track of allocated pages, etc. For now, this is just a simple wrapper around the physical page allocator that returns a pointer to a new page table (which is just a page of memory that we can use for page tables).

use crate::memory::phys::alloc_page;

pub unsafe fn alloc_page_table() -> Option<*mut u64> {
    alloc_page().map(|pa| pa as *mut u64)
}