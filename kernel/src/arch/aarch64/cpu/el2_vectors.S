.section .exceptions_el2, "ax"
.align 11
.global _vector_table_el2

_vector_table_el2:
    // 0x000: sync from current EL with SP0
    b el2_sync_sp0
    .balign 0x80

    // 0x080: IRQ from current EL with SP0
    b el2_irq_sp0
    .balign 0x80

    // 0x100: FIQ from current EL with SP0
    b el2_fiq_sp0
    .balign 0x80

    // 0x180: SError from current EL with SP0
    b el2_serror_sp0
    .balign 0x80

    // 0x200: sync from current EL with SPx (EL2h)
    b el2_sync
    .balign 0x80

    // 0x280: IRQ from current EL with SPx
    b el2_irq
    .balign 0x80

    // 0x300: FIQ from current EL with SPx
    b el2_fiq
    .balign 0x80

    // 0x380: SError from current EL with SPx
    b el2_serror
    .balign 0x80

    // ... 

.global el2_sync_sp0
el2_sync_sp0:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_sync_sp0
    eret

.global el2_irq_sp0
el2_irq_sp0:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_irq_sp0
    eret

.global el2_fiq_sp0
el2_fiq_sp0:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_fiq_sp0
    eret

.global el2_serror_sp0
el2_serror_sp0:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_serror_sp0
    eret

.global el2_sync
el2_sync:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_sync
    eret
    //b .

.global el2_irq
el2_irq:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_irq
    eret

.global el2_fiq
el2_fiq:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_fiq
    eret

.global el2_serror
el2_serror:
    mrs x0, esr_el2
    mrs x1, far_el2
    bl handle_el2_serror
    eret