ENTRY(_start)

SECTIONS
{
    /* kernel loading address */
    . = 0x40080000;

    /* Executable code */
    .text : ALIGN(4K) {
        _text_start = .;
        KEEP(*(.text._start))
        *(.text*)
        _text_end = .;
    }

    /* Align explicitly a specific section .exceptions */
    .exceptions : ALIGN(2048) {
        KEEP(*(.exceptions))
    }

    /* Read only data */
    .rodata : ALIGN(4K) {
        _rodata_start = .;
        *(.rodata*)
        _rodata_end = .;
    }

    /* Initialized data */
    .data : ALIGN(4K) {
        _data_start = .;
        *(.data*)
        _data_end = .;
    }

    /* BSS (zero-initialized) */
    .bss (NOLOAD) : ALIGN(4K) {
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        _bss_end = .;
    }

    /* Stack section */
    .stack (NOLOAD) : ALIGN(16) {
        _stack_start = .;
        . = . + 0x10000;   /* 64 KB stack */
        _stack_top = .;
    }

    /* Kernel global bounds */
    _kernel_start = _text_start;
    _kernel_end   = _bss_end;

}
