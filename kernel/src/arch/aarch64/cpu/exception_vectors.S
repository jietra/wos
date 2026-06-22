// exception_vectors.S

.section .exceptions, "ax"      // use `.section .text, "ax"` if no specific section .exception in linker
.align 11                       // 11 bits (2^11 = 2048 bytes) are RES0, so table has to be aligned to 0x800 bytes
.global exception_vectors       // declare vector to ASM

exception_vectors:              // Base vectors table for EL1
    // -- 01. 0x000: Synchronous exception from current EL with SP0 ------------------
    b sync_current_sp0
    .balign 0x80                // Needed to keep 8 bytes alignment

    // -- 02. 0x080: IRQ/vIRQ exception from current EL with SP0
    b irq_current_sp0
    .balign 0x80

    // -- 03. 0x100 = 0x080 + 0x080: FIQ/vFIQ exception from current EL with SP0
    b fiq_current_sp0
    .balign 0x80

    // -- 04. 0x180 = 0x100 + 0x080: SError/vSError exception from current EL with SP0
    b serr_current_sp0
    .balign 0x80

    // -- 05. 0x200: Synchronous from current EL with SPx ------------------
    b sync_current_spx
    .balign 0x80

    // -- 06. 0x280: IRQ from current EL with SPx
    //b irq_current_spx           // <----- Interrupts ----- //
    b irq_entry
    .balign 0x80

    // -- 07. 0x300: FIQ from current EL with SPx
    b fiq_current_spx
    .balign 0x80

    // -- 08. 0x380: SError from current EL with SPx
    b serr_current_spx
    .balign 0x80

    // -- 09. 0x400: Synchronous from lower EL using AArch64 ------------------
    b sync_lower_64
    .balign 0x80

    // -- 10. 0x480: IRQ from lower EL using AArch64
    b irq_lower_64
    .balign 0x80

    // -- 11. 0x500: FIQ from lower EL using AArch64
    b fiq_lower_64
    .balign 0x80

    // -- 12. 0x580: SError from lower EL using AArch64
    b serr_lower_64
    .balign 0x80

    // -- 13. 0x600: Synchronous from lower EL using AArch32 ------------------
    b sync_lower_32
    .balign 0x80

    // -- 14. 0x680: IRQ from lower EL using AArch32
    b irq_lower_32
    .balign 0x80

    // -- 15. 0x700: FIQ from lower EL using AArch32
    b fiq_lower_32
    .balign 0x80

    // -- 16. 0x780: SError from lower EL using AArch32
    b serr_lower_32
    .balign 0x80

// -----------------------------
// Handlers calling Rust
// -----------------------------

// --- current EL / SP0 --------------------------
.global sync_current_sp0
sync_current_sp0:
    bl sync_current_sp0_rust
    eret

.global irq_current_sp0
irq_current_sp0:
    bl irq_current_sp0_rust
    eret

.global fiq_current_sp0 
fiq_current_sp0:
    bl fiq_current_sp0_rust
    eret

.global serr_current_sp0
serr_current_sp0:
    bl serr_current_sp0_rust
    eret

// --- current EL / SPx --------------------------
.global sync_current_spx
sync_current_spx:
    bl sync_current_spx_rust
    eret

.global irq_current_spx             // <----- Interrupts ----- //
irq_current_spx:
    // Store context (e.g. x0-x17 + lr)
    sub     sp, sp, #(18 * 8)
    stp     x0,  x1,  [sp, #(0 * 16)]
    stp     x2,  x3,  [sp, #(1 * 16)]
    stp     x4,  x5,  [sp, #(2 * 16)]
    stp     x6,  x7,  [sp, #(3 * 16)]
    stp     x8,  x9,  [sp, #(4 * 16)]
    stp     x10, x11, [sp, #(5 * 16)]
    stp     x12, x13, [sp, #(6 * 16)]
    stp     x14, x15, [sp, #(7 * 16)]
    stp     x16, x17, [sp, #(8 * 16)]
    str     x30,      [sp, #(9 * 16)]   // LR

    //bl irq_handler_rust
    bl irq_current_spx_rust

    // Restore context
    ldp     x0,  x1,  [sp, #(0 * 16)]
    ldp     x2,  x3,  [sp, #(1 * 16)]
    ldp     x4,  x5,  [sp, #(2 * 16)]
    ldp     x6,  x7,  [sp, #(3 * 16)]
    ldp     x8,  x9,  [sp, #(4 * 16)]
    ldp     x10, x11, [sp, #(5 * 16)]
    ldp     x12, x13, [sp, #(6 * 16)]
    ldp     x14, x15, [sp, #(7 * 16)]
    ldp     x16, x17, [sp, #(8 * 16)]
    ldr     x30,      [sp, #(9 * 16)]
    add     sp, sp, #(18 * 8)

    eret

.global fiq_current_spx
fiq_current_spx:
    bl fiq_current_spx_rust
    eret

.global serr_current_spx
serr_current_spx:
    bl serr_current_spx_rust
    eret

// --- lower EL / AArch64 --------------------------
.global sync_lower_64
sync_lower_64:
    bl sync_lower_64_rust
    eret

.global irq_lower_64
irq_lower_64:
    bl irq_lower_64_rust
    eret

.global fiq_lower_64
fiq_lower_64:
    bl fiq_lower_64_rust
    eret

.global serr_lower_64
serr_lower_64:
    bl serr_lower_64_rust
    eret

// --- lower EL / AArch32 --------------------------
.global sync_lower_32
sync_lower_32:
    bl sync_lower_32_rust
    eret

.global irq_lower_32
irq_lower_32:
    bl irq_lower_32_rust
    eret

.global fiq_lower_32
fiq_lower_32:
    bl fiq_lower_32_rust
    eret

.global serr_lower_32
serr_lower_32:
    bl serr_lower_32_rust
    eret

// -----------------------------
// IRQ EL1
// -----------------------------

.global irq_entry
irq_entry:
    // 1. Load CURRENT
    adrp    x9, CURRENT
    ldr     x10, [x9, :lo12:CURRENT]      // x10 = current

    // 2. Compute &TASKS[current].ctx
    adrp    x11, TASKS
    add     x11, x11, :lo12:TASKS         // x11 = &TASKS[0]

    mov     x12, x10
    lsl     x12, x12, #8                  // *256
    add     x12, x12, x10, lsl #4         // + *16 = *272 (sizeof each Task in TASKS)
    add     x11, x11, x12                 // x11 = &TASKS[current].ctx

    // 3. Save Task context in ctx (x11 = base)

    // x0 first, while still intact
    str x0, [x11, #0]

    // x1..x7
    stp x1, x2, [x11, #8]   // store pair OK (base+8 aligned)
    stp x3, x4, [x11, #24]  // store pair OK (base+24 aligned)
    stp x5, x6, [x11, #40]
    str x7,     [x11, #56]  // store register to keep 16bytes alignment (base+56 NOT aligned)

    // x8..x15
    stp x8,  x9,  [x11, #64]
    stp x10, x12, [x11, #80] // skip x11 which serves as base
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

    // 4. Call scheduler : next = irq_handle_and_schedule(current)
    mov     x0, x10                       // current
    bl      irq_handle_and_schedule
    mov     x10, x0                       // next

    // 5. CURRENT = next
    adrp    x9, CURRENT
    str     x10, [x9, :lo12:CURRENT]

    // 6. &TASKS[next].ctx
    adrp    x11, TASKS
    add     x11, x11, :lo12:TASKS
    mov     x12, x10
    lsl     x12, x12, #8
    add     x12, x12, x10, lsl #4
    add     x11, x11, x12                 // x11 = &TASKS[next].ctx

    // 7. Restore context from ctx (x11 = base)

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
    ldr x0, [x11,#248]
    mov sp, x0

    // elr
    ldr x0, [x11,#256]
    msr elr_el1, x0

    // spsr
    ldr x0, [x11,#264]
    msr spsr_el1, x0

    // restore x0 last
    ldr x0, [x11,#0]

    eret

.global start_first_task
start_first_task:
    // x0 = &TASKS[0].ctx

    // Load &TASKS[0].ctx
    adrp    x11, TASKS
    add     x11, x11, :lo12:TASKS   // x11 = &TASKS[0]
    mov     x0, x11                 // x0 = &TASKS[0].ctx (ctx at the beginning)

    // --- same code as the restore part in irq_entry ---

    // base = x0
    mov x11, x0

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
    ldr x0, [x11,#248]
    mov sp, x0

    // elr
    ldr x0, [x11,#256]
    msr elr_el1, x0

    // spsr
    ldr x0, [x11,#264]
    msr spsr_el1, x0

    // x0 last
    ldr x0, [x11,#0]

    eret
