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
    b irq_current_spx
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

.global irq_current_spx
irq_current_spx:
    bl irq_current_spx_rust
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