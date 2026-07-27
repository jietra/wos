ENTRY(_start)

PHDRS
{
    kernel      PT_LOAD FLAGS(0x5);     /* RX */
    kernel_data PT_LOAD FLAGS(0x6);     /* RW */
    user        PT_LOAD FLAGS(0x4);     /* R only, or even no flags */
}


SECTIONS
{
    /* kernel loading address on QEMU virt RISC-V */
    . = 0x80000000;

    /* Executable code */
    .text : {
        . = ALIGN(4K);
        _text_start = .;
        KEEP(*(.text._start))
        *(.text*)
        _text_end = .;
    } :kernel

    /* (optional) section for traps/interrupts */
    .exceptions : {
        . = ALIGN(4K);
        KEEP(*(.exceptions))
    } :kernel

    /* Read only data */
    .rodata : {
        . = ALIGN(4K);
        _rodata_start = .;
        *(.rodata*)
        _rodata_end = .;
    } :kernel

    /* Initialized data */
    .data : {
        . = ALIGN(4K);
        _data_start = .;
        *(.data*)
        _data_end = .;
    } :kernel_data

    /* BSS (zero-initialized) */
    .bss (NOLOAD) : {
        . = ALIGN(4K);
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        _bss_end = .;
    } :kernel_data

    /* Kernel stack section */
    .stack (NOLOAD) : {
        . = ALIGN(16);
        _stack_start = .;
        . = . + 0x10000;   /* 64 KiB */
        _stack_top = .;
    } :kernel_data

    /* Kernel global bounds */
    _kernel_start = _text_start;
    _kernel_end   = _bss_end;

    /* ---------------- Userland sections (EL0) ---------------- */

    . = 0x90000000;

    .user_text : {
        . = ALIGN(4K);
        _user_text_start = .;
        *(.user_text*)
        _user_text_end = .;
    } :user

    .user_data : {
        . = ALIGN(4K);
        _user_data_start = .;
        *(.user_data*)
        _user_data_end = .;
    } :user

    .user_stack (NOLOAD) : {
        . = ALIGN(4K);
        _user_stack_start = .;
        . = . + 0x40000;    /* 256 KB stack */
        _user_stack_end = .;
        _user_stack_top = _user_stack_end - 0x10;
    } :user

}
