// src/arch/aarch64/mmio.rs

use crate::memory::memory_layout::layout::DEVICE_BASE;

pub static mut UART_BASE: usize = UART_PADDR;
pub const UART_PADDR: usize = 0x0900_0000;
pub const UART_VADDR: usize = DEVICE_BASE + 0x0020_0000;