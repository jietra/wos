// src/arch/aarch64/boot/linker_symbols.rs

extern "C" {
    // Kernel sections
    pub static _text_start: u8;
    pub static _text_end: u8;

    pub static _rodata_start: u8;
    pub static _rodata_end: u8;

    pub static _data_start: u8;
    pub static _data_end: u8;

    pub static _bss_start: u8;
    pub static _bss_end: u8;

    // Stack
    pub static _stack_start: u8;
    pub static _stack_top: u8;

    // User sections
    pub static _user_text_start: u8;
    pub static _user_text_end: u8;
    
    pub static _user_data_start: u8;
    pub static _user_data_end: u8;

    pub static _user_stack_start: u8;
    pub static _user_stack_top: u8;     // safe top (mapped)
    pub static _user_stack_end: u8;

    pub static _stack_top_el2: u8;

    // Kernel global bounds
    pub static _kernel_start: u8;
    pub static _kernel_end: u8;

    // Exceptions (incl. IRQ etc.)
    pub static _exceptions_start: u8;
    pub static _exceptions_end: u8;

    pub static _heap_start: u8;
    pub static _heap_end: u8;

    // Boot tables
    pub static _boot_tables_start: u8;
    pub static _boot_tables_end: u8;

    // Stage2 tables
    pub static _stage2_start: u8;
    pub static _stage2_end: u8;

    pub static _exceptions_el2_start: u8;
    pub static _exceptions_el2_end: u8;
    pub static _stack_el2_start: u8;
    pub static _stage2_root_start: u8;
    pub static _stage2_root_end: u8;
    pub static _guest_images_start: u8;
    pub static _guest_images_end: u8;

    pub static _page_pool_start: u8;
    pub static _page_pool_end: u8;
}
