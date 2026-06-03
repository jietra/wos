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

1:  // If EL1h
    // Init SP
    msr SPSel, #1           // select SP_EL1 (EL1h) instead of SP0 (EL1t)
    ldr x1, =_stack_top     // declare _stack_top in linker
    mov sp, x1
    bl rust_main            // jump to rust_main

2:
    b 2b
