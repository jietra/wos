ENTRY(_entry)

SECTIONS
{
    /* ------------- Kernel section ------------- */

    /* kernel loading address */
    . = 0x40080000;

    /* Executable code */
    .text : ALIGN(4K) {
        _text_start = .;
        KEEP(*(.text._entry))   /* common entry for EL2/EL1 */
        KEEP(*(.text._hboot))
        KEEP(*(.text._boot))
        *(.text*)
        _text_end = .;
    }

    /* Align explicitly a specific section .exceptions */
    .exceptions : ALIGN(2048) {
        _exceptions_start = .;
        KEEP(*(.exceptions))
        _exceptions_end = .;
    }

    /* EL2 exception vectors */
    .exceptions_el2 : ALIGN(2048) {
        _exceptions_el2_start = .;
        KEEP(*(.exceptions_el2*))
        _exceptions_el2_end = .;
    }

    /* Guest EL1 exception vectors */
    .guest_el1_vectors : ALIGN(0x1000) {
        KEEP(*(.guest_el1_vectors))
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

    /* Page tables must live in identity-mapped RAM */
    .boot_tables : ALIGN(4K) {
        _boot_tables_start = .;
        KEEP(*(.boot_tables))
        _boot_tables_end = .;
    }

    /* Stack section (EL2)*/
    .stack_el2 (NOLOAD) : ALIGN(16) {
        _stack_el2_start = .;
        . = . + 0x10000;
        _stack_top_el2 = .;
    }

    /* Stack section (EL1)*/
    .stack (NOLOAD) : ALIGN(16) {
        _stack_start = .;
        . = . + 0x10000;   /* 64 KB stack */
        _stack_top = .;
    }

    /* Heap / page allocator zone */
    .heap (NOLOAD) : ALIGN(0x1000) {
        _heap_start = .;
        . = . + 0x100000;   /* e.g. 1 MiB heap */
        _heap_end = .;
    }

    /* Page pool section */
    .page_pool (NOLOAD) : ALIGN(4096) {
        _page_pool_start = .;
        . += 0x00800000; /* 8 MiB */
        _page_pool_end = .;
    }

    /* Stage-2 tables (EL2) */
    .stage2 (NOLOAD) : ALIGN(4096) {
        _stage2_start = .;
        . += 0x00600000;   /* 6 MiB: has to be sufficient to support guest kernels */
        _stage2_end = .;
    }

    /* Stage-2 root */
    .stage2_root : ALIGN(4096) {
        _stage2_root_start = .;
        *(.stage2_root)
        _stage2_root_end = .;
    }

    /* Kernel global bounds */
    _kernel_start = _text_start;
    _kernel_end   = .; /*_heap_end;*/

    /* ------------- User section ------------- */

    /*. = 0x41000000;*/
    /* put user section after kernel, with a margin */
    . = ALIGN(4K);
    . = _kernel_end + 0x200000;

    .user_text : ALIGN(4K) {
        _user_text_start = .;
        *(.user_text*)
        _user_text_end = .;
    }

    .user_data : ALIGN(4K) {
        _user_data_start = .;
        *(.user_data*)
        _user_data_end = .;
    }

    /* Stack section (EL0)*/
    .user_stack (NOLOAD) : ALIGN(0x1000) {
        _user_stack_start = .;
        . = . + 0x40000;    /* 256 KB */
        _user_stack_end = .;
        _user_stack_top = _user_stack_end - 0x10; /* safe top */
    }

    /* ------------- Guest section ------------- */
    /* Not used for now */
    .guest_images (NOLOAD) : ALIGN(4K) {
        _guest_images_start = .;
        KEEP(*(.guest_images))
        _guest_images_end = .;
    }


}