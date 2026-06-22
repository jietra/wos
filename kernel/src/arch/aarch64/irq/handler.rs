// src/arch/aarch64/irq/handler.rs

use crate::arch::aarch64::gic::gicv2::gicv2;
use crate::arch::aarch64::timer::cntp::cntp as timer;
use crate::arch::aarch64::uart::pl011;

#[no_mangle]
pub extern "C" fn irq_handle_and_schedule(current: usize) -> usize {

    unsafe {
        let ispend_before = // For debug log
            core::ptr::read_volatile((gicv2::GICD_PADDR + 0x200) as *const u32);

        // read GICC_IAR
        let iar = gicv2::ack();
        let id  = iar & 0x3FF;
        crate::uart_println!("[IRQ] iar=0x{:08x}", iar);
        crate::uart_println!("[IRQ] id ={}", id);

        let next = match id {
            timer::TIMER_IRQ => {
                crate::uart_println!("[IRQ] |TIMER IRQ| current={}", current);
                
                // reload timer
                timer::on_tick();
                
                // change task
                let n = crate::scheduler::task::schedule_next(current);
                crate::uart_println!("[IRQ] |TIMER IRQ| next={}", n);
                n
            }
            pl011::UART_IRQ => {
                crate::uart_println!("[IRQ] |UART IRQ|");

                // UART: no scheduling
                pl011::on_irq();
                current
            }
            _ => {
                // SGI / spurious: debug, no scheduling
                crate::uart_println!("[IRQ] |OTHER IRQ| GICD_ISPENDR0 before ack = 0x{:08x}", ispend_before);
                let ispend0 =
                    core::ptr::read_volatile((gicv2::GICD_PADDR + 0x200) as *const u32);
                crate::uart_println!("[IRQ] |OTHER IRQ| GICD_ISPENDR0 after      = 0x{:08x}", ispend0);

                current
            }
        };

        gicv2::eoi(iar);
        next
    }
}
