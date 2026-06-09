    .section .text._start
    .globl _start
    .globl rust_main

_start:
    # 1. Deactivates interrupts (clean)
    csrw mie, zero
    csrw mip, zero

    # 2. Initialize stack
    la sp, _stack_top

    # 3. Call rust_main()
    call rust_main

1:
    j 1b
