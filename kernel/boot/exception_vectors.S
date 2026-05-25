    .section .text.exceptions, "ax"
    .align 11              // 2^11 = 2048 bytes alignment
    .global exception_vectors

// Base vectors table for EL1
exception_vectors:
    // 0x000: Synchronous exception from current EL with SP0
    b sync_current_sp0

    // 0x080: IRQ from current EL with SP0
    b irq_current_sp0

    // 0x100: FIQ from current EL with SP0
    b fiq_current_sp0

    // 0x180: SError from current EL with SP0
    b serr_current_sp0

    // 0x200: Synchronous exception from current EL with SPx
    b sync_current_spx

    // 0x280: IRQ from current EL with SPx
    b irq_current_spx

    // 0x300: FIQ from current EL with SPx
    b fiq_current_spx

    // 0x380: SError from current EL with SPx
    b serr_current_spx

    // 0x400: Synchronous exception from lower EL using AArch64
    b sync_lower_64

    // 0x480: IRQ from lower EL using AArch64
    b irq_lower_64

    // 0x500: FIQ from lower EL using AArch64
    b fiq_lower_64

    // 0x580: SError from lower EL using AArch64
    b serr_lower_64

    // 0x600: Synchronous exception from lower EL using AArch32
    b sync_lower_32

    // 0x680: IRQ from lower EL using AArch32
    b irq_lower_32

    // 0x700: FIQ from lower EL using AArch32
    b fiq_lower_32

    // 0x780: SError from lower EL using AArch32
    b serr_lower_32

// Minimal Handlers (loops)
sync_current_sp0:  b .
irq_current_sp0:   b .
fiq_current_sp0:   b .
serr_current_sp0:  b .

sync_current_spx:  b .
irq_current_spx:   b .
fiq_current_spx:   b .
serr_current_spx:  b .

sync_lower_64:     b .
irq_lower_64:      b .
fiq_lower_64:      b .
serr_lower_64:     b .

sync_lower_32:     b .
irq_lower_32:      b .
fiq_lower_32:      b .
serr_lower_32:     b .