ENTRY(_start)

SECTIONS {
    . = 0x40000000;

    .text : {
        KEEP(*(.text._start))
        *(.text*)
        *(.init*)
        *(.plt*)
    }

    .rodata : {
        *(.rodata*)
        *(.rodata.*)
    }

    .data : {
        *(.data*)
        *(.data.*)
        *(.got*)
        *(.got.plt*)
        *(.data.rel.ro*)
    }

    .bss : {
        *(.bss*)
        *(.bss.*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.eh_frame_hdr*)
        *(.gcc_except_table*)
        *(.comment*)
        *(.note*)
    }
}
