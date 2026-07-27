// src/arch/aarch64/process/process.rs

use crate::arch::aarch64::boot::linker_symbols as ls;
use crate::scheduler::process::{
    //spawn_kernel_process,
    spawn_user_process,
    CURRENT_PID,
    CTX
};
use crate::scheduler::Context;
use crate::arch::aarch64::mmu::tables::L0_LOW;
//use crate::tasks::{task0_entry, task1_entry, task2_entry};

extern "C" { fn start_first_proc() -> !; }

// ---------------------------------------------------------------------------
// Initiate processes
// ---------------------------------------------------------------------------
pub unsafe fn init_processes() {
    //crate::uart_println!("| INIT. | Initializing scheduler (3 processes)...");
    crate::uart_println!("| INIT. | Initializing scheduler (1 user)...");

    //let p0 = spawn_kernel_process(task0_entry as *const () as usize);
    //let p1 = spawn_kernel_process(task1_entry as *const () as usize);
    //let p2 = spawn_kernel_process(task2_entry as *const () as usize);
    let ps = spawn_user_process(crate::user::shell::user_shell_entry as *const () as usize);

    //CURRENT_PID = p0;
    CURRENT_PID = ps;

    //crate::uart_println!("\tPID0 (kernel): ", p0);
    //crate::uart_println!("\tPID1 (kernel): ", p1);
    //crate::uart_println!("\tPID2 (kernel): ", p2);
    crate::uart_println!("\tPID3 (user)  : ", ps);

    use core::mem::size_of;
    crate::uart_println!("\tsizeof(Context) = {}", size_of::<Context>());
    crate::uart_println!("\tCTX[0].sp  = 0x{:016x}", CTX[0].sp);
    crate::uart_println!("\tCTX[0].pc  = 0x{:016x}", CTX[0].pc);
    //crate::uart_println!("\tCTX[1].sp  = 0x{:016x}", CTX[1].sp);
    //crate::uart_println!("\tCTX[1].pc  = 0x{:016x}", CTX[1].pc);
    //crate::uart_println!("\tCTX[2].sp  = 0x{:016x}", CTX[2].sp);
    //crate::uart_println!("\tCTX[2].pc  = 0x{:016x}", CTX[2].pc);
}

pub unsafe fn start_first_proc_rust() {
    crate::uart_println!("| CHECK | Launching processe(s)...");
    
    crate::uart_println!("\ttext_start       = 0x", &ls::_text_start as *const _ as u64);
    crate::uart_println!("\ttext_end         = 0x", &ls::_text_end   as *const _ as u64);
    crate::uart_println!("\tbss_end          = 0x", &ls::_bss_end    as *const _ as u64);
    crate::uart_println!("\tuser_text_start  = 0x", &ls::_user_text_start as *const _ as u64);
    crate::uart_println!("\tuser_text_end    = 0x", &ls::_user_text_end   as *const _ as u64);
    crate::uart_println!("\tuser_data_start  = 0x", &ls::_user_data_start as *const _ as u64);
    crate::uart_println!("\tuser_data_end    = 0x", &ls::_user_data_end   as *const _ as u64);
    crate::uart_println!("\tuser_stack_start = 0x", &ls::_user_stack_start as *const _ as u64);
    crate::uart_println!("\tuser_stack_top   = 0x", &ls::_user_stack_top as *const _ as u64);

    let va = &ls::_user_stack_top as *const u8 as u64;
    let l0 = ((va >> 39) & 0x1FF) as usize;
    let l1 = ((va >> 30) & 0x1FF) as usize;
    let l2 = ((va >> 21) & 0x1FF) as usize;
    let l3 = ((va >> 12) & 0x1FF) as usize;

    crate::uart_println!("\tuser stack VA = 0x{:016x}", va);
    crate::uart_println!("\tindices: L0={}", l0);
    crate::uart_println!("\tindices: L1={}", l1);
    crate::uart_println!("\tindices: L2={}", l2);
    crate::uart_println!("\tindices: L3={}", l3);

    let l0e = L0_LOW.0[l0];
    crate::uart_println!("\tL0_LOW[l0] = 0x{:016x}", l0e);

    let l1_pa = l0e & !0xFFF;
    let l1_tab = l1_pa as *const u64;
    let l1e = unsafe { *l1_tab.add(l1) };
    crate::uart_println!("\tL1[l1] = 0x{:016x}", l1e);

    let l2_pa = l1e & !0xFFF;
    let l2_tab = l2_pa as *const u64;
    let l2e = unsafe { *l2_tab.add(l2) };
    crate::uart_println!("\tL2[l2] = 0x{:016x}", l2e);

    let l3_pa = l2e & !0xFFF;
    let l3_tab = l3_pa as *const u64;
    let l3e = unsafe { *l3_tab.add(l3) };
    crate::uart_println!("\tL3[l3] = 0x{:016x}", l3e);


    start_first_proc();
}