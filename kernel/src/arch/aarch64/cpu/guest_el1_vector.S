    .section .guest_el1_vectors, "ax"
    .global _guest_el1_vectors

_guest_el1_vectors:
    // Synchronous exception from EL1
    b guest_el1_sync

    // IRQ from EL1
    b guest_el1_irq

    // FIQ from EL1
    b guest_el1_fiq

    // SError from EL1
    b guest_el1_serror


guest_el1_sync:
    // Log ESR_EL1 / FAR_EL1 / ELR_EL1
    mrs x0, esr_el1
    mrs x1, far_el1
    mrs x2, elr_el1

    // Call Rust for print
    bl guest_el1_exception_handler

1:  b 1b

guest_el1_irq:
1:  b 1b

guest_el1_fiq:
1:  b 1b

guest_el1_serror:
1:  b 1b
