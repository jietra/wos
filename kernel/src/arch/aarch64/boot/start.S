// start.S — Kernel EL1 + Hypervisor EL2 bootstrap (lean)

.section .text._entry, "ax"
.global _entry

_entry:
    // --- Read CurrentEL -----------
    mrs x0, CurrentEL
    lsr x0, x0, #2

    cmp x0, #2
    beq _hboot        // EL2: Hypervisorr

    cmp x0, #1
    beq _boot         // EL1: Kernel

0:
    b 0b              // EL0: loop

// EL2h
.section .text._hboot, "ax"
.global _hboot
_hboot:
    // --- Init SP_EL2 -------------------
    msr SPSel, #1           // select SP_EL2 (EL2h)
    isb
    ldr x1, =_stack_top_el2 // declare _stack_top_el2 in linker
    mov sp, x1

    // --- Minimal EL2 setup (lean hypervisor) ---

    // 1. SCTLR_EL2: safe baseline (MMU off for now)
    mrs x0, SCTLR_EL2
    // Can be tweaked later when enabling EL2 MMU
    msr SCTLR_EL2, x0
    isb

    // 2. HCR_EL2: enable virtualization + EL1 AArch64
    // VM = 1 (stage-2 enabled)
    // RW = 1 (EL1 runs in AArch64)
    mov x0, #(1 << 0)       // VM = 1
    orr x0, x0, #(1 << 31)  // RW = 1
    // Later we can add IMO/FMO/AMO for trapping IRQ/FIQ/SError
    msr HCR_EL2, x0
    isb

    // 3. VBAR_EL2: EL2 exception vector
    ldr x0, =_vector_table_el2
    msr VBAR_EL2, x0
    isb

    // 4. Enable FP/SIMD in EL2 (optional)
    // mrs x0, CPTR_EL2
    // bic x0, x0, #(1 << 10)   // clear TFP (trap FP)
    // msr CPTR_EL2, x0
    // isb

    // --- Jump to Rust hypervisor entrypoint ---
    bl rust_main_el2

2:
    b 2b


// Separate data (stack)
.section .data
.global _stack_top_el2
_stack_top_el2:
    .quad 0                 // linker will place actual value

// kernel start.S

.section .text._boot, "ax"
.global _boot

_boot:
    // EL1h
    // Init SP
    msr SPSel, #1           // select SP_EL1 (EL1h) instead of SP0 (EL1t)
    isb
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
