// src/debug/cpu.rs

use crate::drivers::uart::puts;

// --- ARM ---

pub unsafe fn read_current_el() {
    puts("\t| --- Reading current EL... --- |\n");
    let el: u64;
    unsafe {
        core::arch::asm!(
            "mrs {0}, CurrentEL",
            out(reg) el,
            options(nostack, preserves_flags),
        );
        crate::uart_println!("\tRaw CurrentEL = 0x{:x}", el);
        crate::uart_println!("\tDecoded EL    = {}", (el >> 2) & 0b11);
    }
}

// Check which CPU
pub unsafe fn dump_mpidr() {
    crate::uart_println!("\t| --- Check current CPU... --- |");
    let mpidr: u64;
    core::arch::asm!("mrs {0}, MPIDR_EL1", out(reg) mpidr);
    crate::uart_println!("\tMPIDR_EL1 = 0x{:016x}", mpidr);
    let cpu = mpidr & 0xff;
    crate::uart_println!("\tCPU ID    = {}", cpu);
}

pub unsafe fn read_daif() {
    crate::uart_println!("\t| --- Reading DAIF... --- |");
    let before = core::ptr::read_volatile(&_daif_before);
    let after  = core::ptr::read_volatile(&_daif_after);
    crate::uart_println!("\tDAIF before _boot    = 0x",before);
    crate::uart_println!("\tDAIF after  _boot    = 0x",after);

    let mut daif: u64;
    core::arch::asm!("mrs {0}, DAIF", out(reg) daif);
    crate::uart_println!("\tDAIF after start.S   = 0x",daif);

    core::arch::asm!("msr daifclr, #2"); // clear I
    core::arch::asm!("mrs {0}, DAIF", out(reg) daif);
    crate::uart_println!("\tDAIF after daifclr#2 = 0x",daif);

    core::arch::asm!("msr daifclr, #4"); // clear I
    core::arch::asm!("mrs {0}, DAIF", out(reg) daif);
    crate::uart_println!("\tDAIF after daifclr#4 = 0x",daif);
}

extern "C" {
    static _daif_before: u64;
    static _daif_after: u64;
}

// --- RISC-V ---

// The following bridge allows using "puts" in ASM codes
// (e.g. for debugging purposes)
#[no_mangle]
pub extern "C" fn asm_puts(ptr: *const u8) {
    unsafe {
        // Compute the lenght of the C chain
        let mut len = 0;
        let mut p = ptr;
        while *p != 0 {
            len += 1;
            p = p.add(1);
        }

        // Convert into Rust slice
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