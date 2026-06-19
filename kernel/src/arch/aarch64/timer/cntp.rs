// src/arch/aarch64/timer/cntp.rs

pub mod cntp {
    pub const TIMER_IRQ: u32 = 30;

    #[inline(always)]
    unsafe fn write_cntp_tval(val: u64) {
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) val);
    }

    #[inline(always)]
    unsafe fn write_cntp_ctl(val: u64) {
        core::arch::asm!("msr cntp_ctl_el0, {}", in(reg) val);
    }

    pub unsafe fn init() {
        crate::uart_println!("| INIT. | Firing cntp init()...");

        let mut freq: u64 = 0;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);

        let ticks = freq / 10;

        write_cntp_tval(ticks);
        write_cntp_ctl(1);             // ENABLE=1, IMASK=0
    }

    pub unsafe fn on_tick() {
        // reload timer
        let mut freq: u64 = 0;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let ticks = freq / 10;
        write_cntp_tval(ticks);

        crate::time::tick::on_tick();  // increment global counter
        //crate::uart_println!("[TIMER] tick");
    }
}
