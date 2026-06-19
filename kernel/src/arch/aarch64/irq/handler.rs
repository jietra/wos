// src/arch/aarch64/irq/handler.rs

use crate::arch::aarch64::gic::gicv2::gicv2;
use crate::arch::aarch64::timer::cntp::cntp as timer;
use crate::arch::aarch64::uart::pl011;

pub fn handle_irq() {
    //crate::uart_println!("[IRQ] IRQ fired!");

    unsafe {
        // Debug:
        //let ispend_before = core::ptr::read_volatile((gicv2::GICD_PADDR + 0x200) as *const u32);
        //crate::uart_println!("[IRQ] ISPENDR0 before ack = 0x{:08x}", ispend_before);

        let iar = gicv2::ack(); // read GICC_IAR
        let id  = iar & 0x3FF;

        match id {
            timer::TIMER_IRQ => timer::on_tick(),
            pl011::UART_IRQ  => pl011::on_irq(),
            _ => { /* spurious IRQ / ignore */
                crate::uart_println!("[IRQ] handler: iar = 0x{:08x}", iar);
                crate::uart_println!("[IRQ] handler: id  = {}", id);
                let ispend0 = core::ptr::read_volatile((gicv2::GICD_PADDR + 0x200) as *const u32);
                crate::uart_println!("[IRQ] GICD_ISPENDR0 after SGI = 0x{:08x}", ispend0);
            }
        }

        gicv2::eoi(iar); // write GICC_EOIR (end of interrupt)
    }
}


