// src/debug/tests.rs

use crate::drivers::uart::puts;
use crate::utils::print::put_hex_ln;

static TEST: u64 = 0x12345678ABCDEF00;
extern "C" {
    fn _start();
}

pub fn tests() {
    puts("| CHECK | Running some tests...\n");

    puts("\tTesting read_volatile...\n");
    unsafe {
        puts("\t\taddr(TEST) \t= "); put_hex_ln(&TEST as *const _ as u64);
        let v = core::ptr::read_volatile(&TEST);
        puts("\t\tr_vol(TEST) \t= "); put_hex_ln(v);
        let va = _start as *const () as u64;
        puts("\t\tVA(_start) \t= "); put_hex_ln(va);
        let pa: u64;
        core::arch::asm!("adrp {0}, _start", out(reg) pa);
        puts("\t\tPA(_start) \t= "); put_hex_ln(pa);
        let p = va as *const u32;
        let w = core::ptr::read_volatile(p) as u64;
        puts("\t\tr_vol(_start) \t= "); put_hex_ln(w);
    }

    puts("\tTesting BRK instruction and exception handling...\n");
    let spsel: u64;
    unsafe {
        core::arch::asm!("mrs {0}, SPSel", out(reg) spsel);
    }
    puts("\t\tSPSel \t= 0x"); put_hex_ln(spsel);
    let esr: u64;
    let far: u64;
    unsafe {
        core::arch::asm!(
            "mrs {0}, ESR_EL1",
            "mrs {1}, FAR_EL1",
            out(reg) esr,
            out(reg) far,
            options(nostack, preserves_flags),
        );
    }
    puts("\t\tESR_EL1 = 0x"); put_hex_ln(esr);
    puts("\t\tFAR_EL1 = 0x"); put_hex_ln(far);
    /*
    puts("\t\t----- BREAK... -----\n");
    unsafe {
        core::arch::asm!("brk #0");
    }
    puts("\t\tAfter BRK\n");
    */

}
