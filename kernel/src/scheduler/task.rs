// src/scheduler/task.rs

// -------------------------------------------------------------
// structure tasks + separate stacks + minimal CPU context
// -------------------------------------------------------------

// CPU context to be stored/restored
#[repr(C)]
pub struct TaskContext {
    pub x: [u64; 31],   // x0..x30
    pub sp: u64,        // stack pointer
    pub elr: u64,       // return address
    pub spsr: u64,      // saved program status
}

impl TaskContext {      // avoid use of memset (no memset in #nostd)
    pub const fn zeroed() -> Self {
        Self {
            x: [0; 31],
            sp: 0,
            elr: 0,
            spsr: 0,
        }
    }
}

#[repr(C)]
pub struct Task {
    pub ctx: TaskContext,
}

// Put stack_top out of Task to have a clean Task Control Block (TCB)
#[no_mangle]
pub static mut STACK_TOP: [u64; NUM_TASKS] = [0; NUM_TASKS];

#[no_mangle]
pub static mut CURRENT: usize = 0;

// --- init 3 tasks ---

pub const NUM_TASKS: usize = 3;

#[no_mangle]
pub static mut TASKS: [Task; NUM_TASKS] = [
    Task { ctx: TaskContext::zeroed()},//, stack_top: 0 },
    Task { ctx: TaskContext::zeroed()},//, stack_top: 0 },
    Task { ctx: TaskContext::zeroed()},//, stack_top: 0 },
];

#[no_mangle]
pub extern "C" fn schedule_next(current: usize) -> usize {
    //crate::uart_println!("[SCHED] schedule_next(current={:#018x})", current);
    (current + 1) % NUM_TASKS
}

// One stack per task (16KiB for now)
const STACK_SIZE: usize = 16 * 1024;

#[repr(align(16))]
struct AlignedStack([u8; STACK_SIZE]);

static mut STACK_TASK0: AlignedStack = AlignedStack([0; STACK_SIZE]);
static mut STACK_TASK1: AlignedStack = AlignedStack([0; STACK_SIZE]);
static mut STACK_TASK2: AlignedStack = AlignedStack([0; STACK_SIZE]);

use crate::tasks::{task0_entry, task1_entry, task2_entry};

pub unsafe fn init_tasks() {
    use core::mem::size_of;
    crate::uart_println!("| INIT. | Initializing scheduler (3 tasks)...");
    
    crate::uart_println!("\tsizeof(TaskContext) = {}", size_of::<TaskContext>());
    crate::uart_println!("\tsizeof(Task)        = {}", size_of::<Task>());

    STACK_TOP[0] = (&raw const STACK_TASK0.0 as *const u8 as u64) + STACK_SIZE as u64;
    STACK_TOP[1] = (&raw const STACK_TASK1.0 as *const u8 as u64) + STACK_SIZE as u64;
    STACK_TOP[2] = (&raw const STACK_TASK2.0 as *const u8 as u64) + STACK_SIZE as u64;

    TASKS[0].ctx.sp = STACK_TOP[0];
    TASKS[1].ctx.sp = STACK_TOP[1];
    TASKS[2].ctx.sp = STACK_TOP[2];

    TASKS[0].ctx.elr = task0_entry as *const () as u64;
    TASKS[1].ctx.elr = task1_entry as *const () as u64;
    TASKS[2].ctx.elr = task2_entry as *const () as u64;

    TASKS[0].ctx.spsr = 0x5; // EL1h, AArch64
    TASKS[1].ctx.spsr = 0x5;
    TASKS[2].ctx.spsr = 0x5;

    CURRENT = 0;
}

extern "C" {
    fn start_first_task() -> !;
}

pub unsafe fn start_first_task_rust() {
    crate::uart_println!("| CHECK | Launching 3 tasks...");
    start_first_task();
    // no return fn (-> !)
}