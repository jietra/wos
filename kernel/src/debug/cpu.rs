// src/debug/cpu.rs

use crate::drivers::uart::puts;
use crate::utils::print::put_hex_ln;

// --- ARM ---

pub unsafe fn read_current_el() {
    puts("| CHECK | Reading current EL...\n");
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

// --- RISC-V ---

// The following bridge allows using "puts" in ASM codes
// (e.g. for debugging purposes)
#[no_mangle]
pub extern "C" fn asm_puts(ptr: *const u8) {
    unsafe {
        // Trouver la longueur de la chaîne C
        let mut len = 0;
        let mut p = ptr;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }

        // Convertir en slice Rust
        let slice = core::slice::from_raw_parts(ptr, len);
        let s = core::str::from_utf8_unchecked(slice);

        crate::drivers::uart::puts(s);
    }
}

extern "C" { fn trap_entry(); }
pub fn test_trap_entry_direct() {
    unsafe { trap_entry(); }
}

pub fn trigger_fault() {
    unsafe {
        //core::arch::asm!("ecall");

        puts("| CHECK | Exception handling...\n");
        core::arch::asm!(".word 0xffffffff");
        puts("\tException handled successfully.\n");
        
        /*
        // Misalignement will be addressed with MMU
        puts("Before misaligned\n");
        let p = 1 as *mut u32;
        core::ptr::read_volatile(p);
        puts("After misaligned\n");
        */
    }
}