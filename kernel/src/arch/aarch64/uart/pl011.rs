// src/arch/aarch64/uart/pl011.rs

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

pub struct VirtPl011 {
    pub tx_fifo: [u8; 16],
    pub tx_head: usize,
    pub tx_tail: usize,

    pub dr:    u32, // Data Register
    pub fr:    u32, // Flag Register
    pub cr:    u32, // Control Register
    pub lcrh:  u32, // Line Control
    pub ibrd:  u32, // Integer Baud
    pub fbrd:  u32, // Fractional Baud,
    pub imsc:  u32, // Interrupt Mask Set/Clear
    pub mis:   u32, // Masked Interrupt Status
    pub icr:   u32, // Interrupt Clear Register
}

impl VirtPl011 {
    pub const fn new() -> Self {
        Self {
            tx_fifo: [0; 16],
            tx_head: 0,
            tx_tail: 0,
            dr:   0,
            // TXFE=1 (bit 7), RXFE=1 (bit 6) -> FIFO empty
            fr:   0xC0,
            cr:   0,
            lcrh: 0,
            ibrd: 0,
            fbrd: 0,
            imsc: 0,
            mis:  0,
            icr:  0,
        }
    }

    fn update_fr(&mut self) {
        let tx_empty = self.tx_head == self.tx_tail;
        self.fr = 0;
        if tx_empty { self.fr |= 1 << 7; } // TXFE
        self.fr |= 1 << 6; // RXFE always empty (no RX)
    }

    fn write_dr(&mut self, val: u32) {
        let c = (val & 0xFF) as u8;

        // push into FIFO
        self.tx_fifo[self.tx_head] = c;
        self.tx_head = (self.tx_head + 1) % 16;

        // if FIFO full, overwrite (Linux never reads RX)
        if self.tx_head == self.tx_tail {
            self.tx_tail = (self.tx_tail + 1) % 16;
        }

        // send to host console
        crate::drivers::uart::putc(c);

        self.update_fr();
    }

    pub fn pl011_mmio_read(&mut self, offset: u64) -> u32 {
        // Update FR before sending
        self.update_fr();
        match offset {
            0x00 => self.dr,
            0x18 => self.fr,
            0x24 => self.ibrd,
            0x28 => self.fbrd,
            0x2C => self.lcrh,
            0x30 => self.cr,
            0x38 => self.imsc,
            0x40 => self.mis,
            0x44 => { self.mis = 0; 0 }, // ICR
            _ => 0,
        }
    }

    pub fn pl011_mmio_write(&mut self, offset: u64, value: u32) {
        match offset {
            0x00 => self.write_dr(value),
            0x18 => { self.fr = value; },
            0x24 => { self.ibrd = value; },
            0x28 => { self.fbrd = value; },
            0x2C => { self.lcrh = value; },
            0x30 => { self.cr = value; },
            0x38 => { self.imsc = value; },
            0x40 => { self.mis = value; },
            0x44 => self.mis = 0,           // clear interrupts
            _ => {},
        }
        self.update_fr();
    }

}

pub static mut VIRT_UART: VirtPl011 = VirtPl011::new();