pub const UART_IRQ: u32 = 33; // QEMU virt: UART0 IRQ = 33

const UART_BASE: usize = 0x0900_0000;

const UART_DR: usize = UART_BASE + 0x00;
const UART_FR: usize = UART_BASE + 0x18;
const UART_IMSC: usize = UART_BASE + 0x38;
const UART_ICR: usize = UART_BASE + 0x44;

#[inline(always)]
unsafe fn mmio_read32(addr: usize) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

#[inline(always)]
unsafe fn mmio_write32(addr: usize, val: u32) {
    core::ptr::write_volatile(addr as *mut u32, val);
}

pub unsafe fn init() {
    // Enable RX interrupt
    mmio_write32(UART_IMSC, 1 << 4);
}

pub unsafe fn on_irq() {
    let c = mmio_read32(UART_DR) as u8;
    crate::uart_println!("{}", c as char);

    // Clear interrupt
    mmio_write32(UART_ICR, 1 << 4);
}
