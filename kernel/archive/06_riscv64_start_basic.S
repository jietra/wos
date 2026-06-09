    .section .text._start
    .globl _start
_start:
    la sp, _stack_top

    li t0, 0x10000000
    li t1, 'X'
    sb t1, 0(t0)

1:  j 1b
