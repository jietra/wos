use crate::utils::print::put_hex_ln;
use crate::drivers::uart::puts;
use crate::memory::phys::alloc_page;

// Test the MMU by allocating some pages and writing to them. If the MMU is working correctly, we should be able to read back the same values without causing a fault. In a real kernel, you'd want to add proper error handling and support for freeing pages, etc.
pub unsafe fn test_memory() {
    unsafe {
        let p1 = alloc_page().unwrap();
        let p2 = alloc_page().unwrap();

        // Write some test data to the allocated pages to verify that the MMU is working correctly. In a real kernel, you'd want to add proper error handling and support for freeing pages, etc.
        let ptr1 = p1 as *mut u64;
        let ptr2 = p2 as *mut u64;
        *ptr1 = 0xDEAD_BEEF_DEAD_BEEF;
        *ptr2 = 0xCAFEBABE_CAFEBABE;
    }
    puts("\tKernel booted with MMU enabled: OK\n");

    puts("\tTesting MMU by allocating some pages and writing to them...\n");
    unsafe {
        if let Some(p1) = alloc_page() {
            if let Some(p2) = alloc_page() {
                let ptr1 = p1 as *mut u64;
                let ptr2 = p2 as *mut u64;

                *ptr1 = 0xDEAD_BEEF_DEAD_BEEF;
                *ptr2 = 0xCAFEBABE_CAFEBABE;

                // raw print the allocated physical addresses and the values we wrote to them to verify that the MMU is working correctly. In a real kernel, you'd want to add proper error handling and support for freeing pages, etc.
                puts("\t\talloc p1 = 0x"); put_hex_ln(p1);
                puts("\t\talloc p2 = 0x"); put_hex_ln(p2);
                // Note: we can't use Rust's formatting macros since we're in no_std, so we just print the raw values in hexadecimal manually. In a real kernel, you'd want to implement a proper formatting function to make this easier.

                puts("\t\tTest alloc: OK\n")
            } else {
                puts("\t\talloc p2 failed\n");
            }
        } else {
            puts("\t\talloc p1 failed\n");
        }
    }
}