// src/scheduler/context.rs

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Context {
    pub regs: [usize; 31],  // generic (31 general registers) (x0..x30 in ARM64 / x1..x31 in RISC-V since x0 hardwired to 0)
    pub sp  : u64,          // Stack Pointer
    pub pc  : usize,        // Program Counter (ELR_EL1  / sepc)
    pub psr : u64,          // Program Status  (SPSR_EL1 / sstatus)
}

impl Context {
    pub const fn zeroed() -> Self {
        Self {
            regs:  [0; 31],
            sp:     0,
            pc:     0,
            psr:    0,
        }
    }
}