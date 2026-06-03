// start.S

.section .text._start, "ax"
.global _start

_start:
    // --- Read CurrentEL -----------
    mrs x0, CurrentEL
    lsr x0, x0, #2          // bits [3:2]
    cmp x0, #1              // EL1 ?
    beq 1f                  // if EL1 then go

    // If current EL = EL0: we cannot directly escalate to EL1
    // In this case, we loop/break
0:
    b 0b

1:  // EL1h
    // Init SP
    msr SPSel, #1           // select SP_EL1 (EL1h) instead of SP0 (EL1t)
    ldr x1, =_stack_top     // declare _stack_top in linker
    mov sp, x1

    // --- Enable FP/SIMD early (avoid FP traps) ---
    // Enable FP/SIMD in CPACR_EL1 to allow use of floating-point and SIMD instructions
    // Required for codes such as read_volatile etc.
    // (LLVM may indeed generate FP/SIMD instructions for some operations,
    // even if we don't explicitly use them in our Rust code - e.g. 
    // for mathematical operations, or even for some memory access patterns).
    mrs x0, CPACR_EL1
    orr x0, x0, #(0b11 << 20)   // FPEN = 0b11 (enable FP/SIMD)
    msr CPACR_EL1, x0
    isb

    // jump to rust_main
    bl rust_main

2:
    b 2b
