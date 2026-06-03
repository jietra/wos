use crate::drivers::uart::puts;
use crate::utils::print::put_hex_ln;

pub unsafe fn read_current_el() {
    puts("| DEBUG | Reading current EL...\n");
    let el: u64;
    unsafe {
        core::arch::asm!(
            "mrs {0}, CurrentEL",
            out(reg) el,
            options(nostack, preserves_flags),
        );
    }
    puts("\tCurrentEL \t= 0x"); put_hex_ln(el);
}