// src/scheduler/process.rs

use crate::scheduler::Context;

#[cfg(target_arch = "aarch64")]
const DEFAULT_PSR: u64 = 0x5;

#[cfg(target_arch = "riscv64")]
const DEFAULT_PSR: u64 = 0;

// ---------------------------------------------------------------------------
// Global parameters and variable
// ---------------------------------------------------------------------------
pub const   MAX_PROCS  : usize = 16;            // max number of processes
const       KSTACK_SIZE: usize = 16 * 1024;     // size of kernel stack <--- limited to 16KiB for now (need to upgrade MMU first)

#[no_mangle]
pub static mut CURRENT_PID: usize = 0;

// ---------------------------------------------------------------------------
// Processes and stacks tables
// ---------------------------------------------------------------------------
// Static table of processes
#[no_mangle]
pub static mut PROCS     : [Option<Process>; MAX_PROCS] = [const { None }; MAX_PROCS];

// Aligned memory block (to store a process stack)
#[derive(Copy, Clone)]
#[repr(align(16))]                          // 16KiB alignment required (ARM64, RISCV64, x86_64)
struct AlignedKStack([u8; KSTACK_SIZE]);    // each stack aligned at KSTACK_SIZE

// Static table of stacks (one stack per process)
#[no_mangle]
pub static mut KSTACK    : [AlignedKStack  ; MAX_PROCS] = [const { AlignedKStack([0; KSTACK_SIZE]) }; MAX_PROCS];

// Keep track of KSTACK_TOP in a separate table in order to keep PCB clean
// Static table of stack tops
#[no_mangle]
pub static mut KSTACK_TOP: [u64            ; MAX_PROCS] = [0; MAX_PROCS];

// Static table of contexts
// Keep track of Contexts in a separate table in order to avoid dealing with too much offsets
#[no_mangle]
pub static mut CTX: [Context; MAX_PROCS] = [Context::zeroed(); MAX_PROCS];

// ---------------------------------------------------------------------------
// Process structure
// ---------------------------------------------------------------------------
#[derive(Copy, Clone)]
#[derive(PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Dead,
}

// PCB (Process Control Block)
pub struct Process {
    pub ctx       : Context,
    pub pid       : usize,
    pub state     : ProcessState,
    pub kstack_top: u64,                 // kernel stack
    pub ustack_top: u64,                 // user stack (for EL0)
    pub mmu_root  : u64,                 // for TTBR0 / satp / etc.
}

// ---------------------------------------------------------------------------
// Kernel process spawner
// ---------------------------------------------------------------------------
pub unsafe fn spawn_kernel_process(entry: usize) -> usize {
    unsafe {
        if let Some(pid) = alloc_pid() {    // allocate an empty slot pid

            // Compute stack top
            let kstack_ptr = KSTACK[pid].0.as_ptr() as u64;
            let kstack_top = kstack_ptr + KSTACK_SIZE as u64;
            let ctx = Context {
                regs: [0; 31],
                sp: kstack_top,
                pc: entry,
                psr: DEFAULT_PSR,
            };

            // Fill in KSTACK_TOP
            KSTACK_TOP[pid] = kstack_top;

            // Fill in CTX
            CTX[pid] = ctx;

            // Create PCB (process control block)
            PROCS[pid] = Some(Process {
                pid,
                state     : ProcessState::Ready,
                ctx       : ctx,
                kstack_top,
                ustack_top: 0,
                mmu_root  : 0,
            });
            
            return pid;
        }

        // When no empty slot
        crate::println!("[PROC] no slot available for process");
        0   //no error handling at this stage
    }
}

// helper function to look for an empty slot and return its pid
fn alloc_pid() -> Option<usize> {
    for pid in 0..MAX_PROCS {
        unsafe {
            if PROCS[pid].is_none() {
                return Some(pid);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Schedule next process
// ---------------------------------------------------------------------------
pub fn schedule_process(current_pid: usize) -> usize {
    for offset in 1..=MAX_PROCS {
        let pid = (current_pid + offset) % MAX_PROCS;
        unsafe {
            if let Some(proc) = &PROCS[pid] {
                if proc.state == ProcessState::Ready {
                    return pid;
                }
            }
        }
    }
    current_pid
}

// ---------------------------------------------------------------------------
// For future use: schedule yield...
// ---------------------------------------------------------------------------
/*
#[no_mangle]
pub extern "C" fn get_context(pid: usize) -> *mut Context {
    unsafe { &mut CTX[pid] }
}

#[no_mangle]
pub extern "C" fn context_switch(old: *mut Context, new: *const Context) {
    // implemented in ASM in switch.S
}

pub fn sched_yield() {
    unsafe {
        let old     = get_context(CURRENT_PID);
        let next    = schedule_process(CURRENT_PID);
        CURRENT_PID = next;
        let new     = get_context(next);
        context_switch(old, new);
    }
}
*/

// ---------------------------------------------------------------------------
// Initiate 3 processes
// ---------------------------------------------------------------------------
//use crate::scheduler::process::spawn_kernel_process;
use crate::tasks::{task0_entry, task1_entry, task2_entry};

pub unsafe fn init_processes() {
    crate::uart_println!("| INIT. | Initializing scheduler (3 processes)...");

    let p0 = spawn_kernel_process(task0_entry as usize);
    let p1 = spawn_kernel_process(task1_entry as usize);
    let p2 = spawn_kernel_process(task2_entry as usize);

    CURRENT_PID = p0;
    crate::uart_println!("\tPID0: ", p0);
    crate::uart_println!("\tPID1: ", p1);
    crate::uart_println!("\tPID2: ", p2);
}

extern "C" {
    fn start_first_proc() -> !;
}

pub unsafe fn start_first_proc_rust() {
    crate::uart_println!("| CHECK | Launching 3 processes...");
    start_first_proc();
}