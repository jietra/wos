use core::arch::asm;
use crate::drivers::uart::{puts};
use crate::uart_println;
extern "C" { fn trap_entry(); }

pub fn init_exceptions() {
    let mstatus: usize;
    unsafe { asm!("csrr {0}, mstatus", out(reg) mstatus); }
    puts("| CHECK | mstatus           = ");
    crate::utils::print::put_hex_ln(mstatus as u64);

    // CHECK Mode
    let mpp = (mstatus >> 11) & 0b11;
    puts("| CHECK | MPP               = ");
    crate::utils::print::put_hex_ln(mpp as u64);

    unsafe {
        let addr = trap_entry as usize;
        puts("| CHECK | trap_entry symbol = ");
        crate::utils::print::put_hex_ln(addr as u64);

        asm!(
            "csrw mtvec, {0}",
            "csrr {1}, mtvec",
            in(reg) addr,
            out(reg) _,
            options(nostack, preserves_flags)
        );

        let mtvec: usize;
        asm!("csrr {0}, mtvec", out(reg) mtvec);
        puts("| CHECK | mtvec CSR         = ");
        crate::utils::print::put_hex_ln(mtvec as u64);
    }
}

#[no_mangle]
pub extern "C" fn trap_handler(tf: *mut u8) {
    uart_println!("\t>>> TRAP HANDLER ACTIVATION <<<");

    let mcause: usize;
    let mepc: usize;
    let mtval: usize;

    unsafe {
        asm!("csrr {0}, mcause", out(reg) mcause);
        asm!("csrr {0}, mepc",   out(reg) mepc);
        asm!("csrr {0}, mtval",  out(reg) mtval);
    }

    uart_println!("\t>>> TRAP <<<");
    uart_println!("\t\tmcause = ", mcause);
    uart_println!("\t\tmepc   = ", mepc);
    uart_println!("\t\tmtval  = ", mtval);

    let is_interrupt = mcause >> (core::mem::size_of::<usize>() * 8 - 1);
    let code = mcause & ((1 << (core::mem::size_of::<usize>() * 8 - 1)) - 1);

    if is_interrupt == 1 {
        handle_interrupt(code, mepc, mtval);
    } else {
        handle_exception(code, mepc, mtval);
    }
}

fn handle_exception(code: usize, mut mepc: usize, mtval: usize) {
    //uart_println!("EXCEPTION code={} mepc={} mtval={}", code, mepc, mtval);
    uart_println!("\tEXCEPTION CAUGHT:");
    uart_println!("\t\t  code=", code as u64);
    uart_println!("\t\t  mepc=", mepc as u64);
    uart_println!("\t\t  mtval=", mtval as u64);
    
    // Skip faulty instruction
    mepc += 4;
    unsafe { asm!("csrw mepc, {0}", in(reg) mepc); }
}

fn handle_interrupt(code: usize, mepc: usize, _mtval: usize) {
    puts("\tINTERRUPT code={} mepc={}");
}
