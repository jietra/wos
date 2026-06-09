ENTRY(_start)

SECTIONS {
    . = 0x80000000;

    .text : ALIGN(4K) {
        KEEP(*(.text._start))
        *(.text*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata*)
    }

    .data : ALIGN(4K) {
        *(.data*)
    }

    .bss (NOLOAD) : ALIGN(16) {
        *(.bss*)
        *(COMMON)

        /* Pile du noyau dans .bss */
        _stack_start = .;
        . = . + 0x10000;   /* 64 KiB */
        _stack_top = .;
    }
}
