// src/arch/aarch64/cpu/switch.S
// For future use

.global context_switch
context_switch:         // called from Rust when NOT IRQ
    // x0 = old: *mut Context
    // x1 = new: *const Context

    // -------------------------
    // 1. Save current CPU state into *old
    // -------------------------

    // base_old = x0
    mov x11, x0

    // x0 first, while still intact
    str x0, [x11, #0]

    // x1..x7
    stp x1, x2, [x11, #8]
    stp x3, x4, [x11, #24]
    stp x5, x6, [x11, #40]
    str x7,     [x11, #56]

    // x8..x15
    stp x8,  x9,  [x11, #64]
    stp x10, x12, [x11, #80]   // x11 skipped since already used as base
    stp x13, x14, [x11, #96]
    stp x15, x16, [x11,#112]

    // x17..x24
    stp x17, x18, [x11,#128]
    stp x19, x20, [x11,#144]
    stp x21, x22, [x11,#160]
    stp x23, x24, [x11,#176]

    // x25..x30
    stp x25, x26, [x11,#192]
    stp x27, x28, [x11,#208]
    stp x29, x30, [x11,#224]

    // sp
    mov x0, sp
    str x0, [x11,#248]

    // elr
    mrs x0, elr_el1
    str x0, [x11,#256]

    // spsr
    mrs x0, spsr_el1
    str x0, [x11,#264]

    // -------------------------
    // 2. Restore CPU state from *new
    // -------------------------

    // base_new = x1
    mov x11, x1

    // x1..x7
    ldp x1, x2, [x11, #8]
    ldp x3, x4, [x11, #24]
    ldp x5, x6, [x11, #40]
    ldr x7,     [x11, #56]

    // x8..x15
    ldp x8,  x9,  [x11, #64]
    ldp x10, x12, [x11, #80]
    ldp x13, x14, [x11, #96]
    ldp x15, x16, [x11,#112]

    // x17..x24
    ldp x17, x18, [x11,#128]
    ldp x19, x20, [x11,#144]
    ldp x21, x22, [x11,#160]
    ldp x23, x24, [x11,#176]

    // x25..x30
    ldp x25, x26, [x11,#192]
    ldp x27, x28, [x11,#208]
    ldp x29, x30, [x11,#224]

    // sp
    ldr x2, [x11,#248]
    mov sp, x2

    // elr
    ldr x2, [x11,#256]
    msr elr_el1, x2

    // spsr
    ldr x2, [x11,#264]
    msr spsr_el1, x2

    // x0 last (regs[0])
    ldr x0, [x11,#0]

    ret     //not eret
