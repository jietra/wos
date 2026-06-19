.section .exceptions, "ax"
.align 11
.global exception_vectors

exception_vectors:
    // 0x000: sync SP0
    b .
    .balign 0x80

    // 0x080: IRQ SP0
    b .
    .balign 0x80

    // 0x100: FIQ SP0
    b .
    .balign 0x80

    // 0x180: SError SP0
    b .
    .balign 0x80

    // 0x200: sync SPx
    b .
    .balign 0x80

    // 0x280: IRQ SPx
    b irq_handler
    .balign 0x80

    // le reste peut rester en `b .` pour l’instant
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80

irq_handler:
    ldr x0, =0x09000000      // UART phys (sans MMU)
    mov x1, #'I'
    str w1, [x0, #0x00]      // DR = 'I'
1:  b 1b
