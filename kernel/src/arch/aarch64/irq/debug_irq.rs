// arch/aarch64/irq/debug_irq.rs

use crate::arch::aarch64::gic::gicv2::gicv2;

pub unsafe fn sgi_irq() {
        crate::uart_println!("| CHECK | Checking SGI O IRQ handling...");
        const GICD_SGIR: usize = 0xF00;
        let val: u32 = 
            (1 << 15) | // NSATT = 1 (SGI Non-secure)
            (1 << 16) | // CPUTargetList = 1 (CPU0)
            0;          // SGI ID = 0
        
        core::ptr::write_volatile((gicv2::GICD_PADDR + GICD_SGIR) as *mut u32, val);
    }