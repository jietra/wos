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

    // Kernel global bounds
    pub static _kernel_start: u8;
    pub static _kernel_end: u8;

    // Exceptions (incl. IRQ etc.)
    pub static _exceptions_start: u8;
    pub static _exceptions_end: u8;
}
