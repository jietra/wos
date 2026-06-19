// start.S

.section .text._boot, "ax"
.global _boot

_boot:
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

    // --- Force DAIF.I = 0 in EL1 -------------------
    mrs x0, DAIF          // read DAIF
    // If I is at 1, we clear with daifclr
    // (bit I = 1 << 7)
    // We touch nothing else.
    msr daifclr, #2
    // Reread DAIF
    mrs x1, DAIF
    // Save x0 and x1 in global symbols
    adrp x2, _daif_before
    add  x2, x2, :lo12:_daif_before
    str  x0, [x2]
    adrp x3, _daif_after
    add  x3, x3, :lo12:_daif_after
    str  x1, [x3]

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

0:
    b 0b

// Separate data (to read DAIF)
.section .data
.global _daif_before
.global _daif_after
_daif_before:
    .quad 0
_daif_after:
    .quad 0