// src/arch/aarch64/gic/gicv2.rs

pub mod gicv2 {
    use crate::memory::memory_layout::layout::DEVICE_BASE;

    /// Physical addresses QEMU virt (GICv2)
    pub const GICD_PADDR: usize = 0x0800_0000;
    pub const GICC_PADDR: usize = 0x0801_0000;

    /// Virtual addresses chosen in the DEVICE zone
    pub const GICD_VADDR: usize = DEVICE_BASE + 0x0000_0000;
    pub const GICC_VADDR: usize = DEVICE_BASE + 0x0001_0000;

    // Using PA temporarily
    pub const GICD_BASE: usize = GICD_PADDR;        // will be moved to GICD_VADDR;
    pub const GICC_BASE: usize = GICC_PADDR;        // will be moved to GICC_VADDR;

    // Main registers offsets (GICv2)
    const GICD_CTLR      : usize = 0x000;
    const GICD_ISENABLER0: usize = 0x100;

    const GICC_CTLR: usize = 0x0000;
    const GICC_PMR : usize = 0x0004;
    const GICC_IAR : usize = 0x000C;
    const GICC_EOIR: usize = 0x0010;

    #[inline(always)]
    unsafe fn mmio_write32(addr: usize, value: u32) {
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    #[inline(always)]
    pub unsafe fn mmio_read32(addr: usize) -> u32 {
        core::ptr::read_volatile(addr as *const u32)
    }

    /// To be called AFTER MMU mapped GICD/GICC in DEVICE_BASE.
    pub unsafe fn init() {
        /*
        /// For debugging purposes:
        use crate::drivers::uart::UART_VADDR;
        crate::uart_println!("gicd_base = 0x{:016x}", GICD_BASE);
        crate::uart_println!("gicc_base = 0x{:016x}", GICC_BASE);
        crate::uart_println!("L0 index for DEVICE_BASE = {}", (DEVICE_BASE >> 39) & 0x1FF);
        crate::uart_println!("L0 index for GICD_VADDR  = {}", (GICD_VADDR >> 39) & 0x1FF);
        crate::uart_println!("L0 index for UART_VADDR  = {}", (UART_VADDR >> 39) & 0x1FF);
        */

        // Everything in group0 secure
        mmio_write32(GICD_BASE + 0x080, 0x0000_0000);  // IGROUPR0 //vs mmio_write32(GICD_BASE + 0x080, 0xFFFF_FFFF); for Group1NS (non secure)
        mmio_write32(GICD_BASE + GICD_CTLR, 0x1);      // Activate Distributor; bit0 = EnableGrp0 //vs mmio_write32(GICD_BASE + GICD_CTLR, 0x2);
        mmio_write32(GICC_BASE + GICC_PMR, 0xFF);      // Activate CPU interface max priority
        mmio_write32(GICC_BASE + GICC_CTLR, 0x1);      // bit0 = EnableGrp0 //vs mmio_write32(GICC_BASE + GICC_CTLR, 0x2);
        /*
        /// For later: Everything in Group1NS (non-secure)
        mmio_write32(GICD_BASE + 0x080, 0xFFFF_FFFF);  // IGROUPR0 : All IRQs in Group1
        mmio_write32(GICD_BASE + GICD_CTLR, 0x2);      // bit1 = EnableGrp1
        mmio_write32(GICC_BASE + GICC_PMR, 0xFF);      // max priority
        mmio_write32(GICC_BASE + GICC_CTLR, 0x2);      // bit1 = EnableGrp1
        */
        
        // Activate all IRQs 0–31 (SGI + PPI)
        mmio_write32(GICD_BASE + GICD_ISENABLER0, 0xFFFF_FFFF);

        enable_irq(30);                                 // Activate timer's IRQ
        //enable_irq(33);                               // Activate UART's IRQ

        /*
        /// For debugging purposes: SGI 0, target CPU0
        const GICD_SGIR: usize = 0xF00;
        let sgi = (0 << 24)   // TargetListFilter = 0 (use CPUTargetList)
                | (1 << 16)   // CPUTargetList = 0b0000_0001 (CPU0)
                | (0 << 0);   // SGI ID = 0
        crate::uart_println!("\tSending SGI 0 via GICD_SGIR...");
        mmio_write32(GICD_BASE + GICD_SGIR, sgi);
        let ispend0 = mmio_read32(GICD_BASE + 0x200); // ISPENDR0
        crate::uart_println!("\tGICD_ISPENDR0 after SGI = 0x{:08x}", ispend0);
        */

    }

    // Activate a given IRQ (global number)
    pub unsafe fn enable_irq(irq: u32) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        let addr = GICD_BASE + GICD_ISENABLER0 + reg * 4;
        let val = 1u32 << bit;
        mmio_write32(addr, val);
    }

    // Read current IRQ (IAR)
    pub unsafe fn ack() -> u32 {
        mmio_read32(GICC_BASE + GICC_IAR)
    }

    // Signal end of IRQ handling
    pub unsafe fn eoi(irq: u32) {
        mmio_write32(GICC_BASE + GICC_EOIR, irq);
    }

    /// For debugging purposes:
    pub unsafe fn dump_gic() {
        let d_ctlr    = mmio_read32(GICD_BASE + GICD_CTLR);
        let d_igroup0 = mmio_read32(GICD_BASE + 0x080);     // IGROUPR0
        let d_isen0   = mmio_read32(GICD_BASE + 0x100);     // ISENABLER0
        let d_ispend0 = mmio_read32(GICD_BASE + 0x200);     // ISPENDR0

        let c_ctlr = mmio_read32(GICC_BASE + GICC_CTLR);
        let c_pmr  = mmio_read32(GICC_BASE + GICC_PMR);
        let c_bpr  = mmio_read32(GICC_BASE + 0x0008);       // GICC_BPR
        let c_rpr  = mmio_read32(GICC_BASE + 0x0014);       // GICC_RPR

        crate::uart_println!("\tGICD_CTLR       = 0x{:08x}", d_ctlr);
        crate::uart_println!("\tGICD_IGROUPR0   = 0x{:08x}", d_igroup0);
        crate::uart_println!("\tGICD_ISENABLER0 = 0x{:08x}", d_isen0);
        crate::uart_println!("\tGICD_ISPENDR0   = 0x{:08x}", d_ispend0);

        crate::uart_println!("\tGICC_CTLR   = 0x{:08x}", c_ctlr);
        crate::uart_println!("\tGICC_PMR    = 0x{:08x}", c_pmr);
        crate::uart_println!("\tGICC_BPR    = 0x{:08x}", c_bpr);
        crate::uart_println!("\tGICC_RPR    = 0x{:08x}", c_rpr);

        let iar = core::ptr::read_volatile((GICC_PADDR + 0x000C) as *const u32);
        crate::uart_println!("\tGICC_IAR raw = 0x{:08x}", iar);
    }
}