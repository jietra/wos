// src/arch/aarch64/timer/cntp.rs

pub mod cntp {
    pub const TIMER_IRQ: u32 = 30;
    static mut FREQ: u64 = 62_500_000;  // default value to be overrid in init()
    static mut TICKS: u64 = 62_500_000; // default value to be overrid in init()
    pub const TIMER_FREQ: u64 = 1;      // 1Hz (chosen for log readability)

    #[inline(always)]
    unsafe fn write_cntp_tval(val: u64) {
        core::arch::asm!("msr cntp_tval_el0, {}", in(reg) val);
    }

    #[inline(always)]
    unsafe fn write_cntp_ctl(val: u64) {
        core::arch::asm!("msr cntp_ctl_el0, {}", in(reg) val);
    }

    pub unsafe fn init() {
        crate::uart_println!("| INIT. | Init cntp...");
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) FREQ);
        crate::uart_println!("\tFREQ=",FREQ);
        TICKS = FREQ / TIMER_FREQ;

        write_cntp_tval(TICKS); // set "ticks" at cntfrq_el0 frequency i.e. frequency at 1 Hz (use (FREQ/10) for 10Hz etc.)
        write_cntp_ctl(1);     // ENABLE=1, IMASK=0
    }

    pub unsafe fn on_tick() {
        write_cntp_tval(TICKS); // reload timer (at 1Hz here)
    }
}
