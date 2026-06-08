// kernel/src/arch/aarch64/interrupts/gicv2.rs

pub mod gicv2 {
    use crate::memory::memory_layout::layout::DEVICE_BASE;

    /// Physical addresses QEMU virt (GICv2)
    const GICD_PADDR: usize = 0x0800_0000;
    const GICC_PADDR: usize = 0x0801_0000;

    /// Virtual addresses chosen in the DEVICE zone
    const GICD_VADDR: usize = DEVICE_BASE + 0x0000_0000;
    const GICC_VADDR: usize = DEVICE_BASE + 0x0001_0000;

    // For now, we use the physical addresses...
    const GICD_BASE: usize = GICD_PADDR;
    const GICC_BASE: usize = GICC_PADDR;

    // Main registers offsets (GICv2)
    const GICD_CTLR: usize      = 0x000;
    const GICD_ISENABLER0: usize = 0x100;

    const GICC_CTLR: usize = 0x0000;
    const GICC_PMR: usize  = 0x0004;
    const GICC_IAR: usize  = 0x000C;
    const GICC_EOIR: usize = 0x0010;

    #[inline(always)]
    unsafe fn mmio_write32(addr: usize, value: u32) {
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    #[inline(always)]
    unsafe fn mmio_read32(addr: usize) -> u32 {
        core::ptr::read_volatile(addr as *const u32)
    }

    /// To be called AFTER MMU mapped GICD/GICC in DEVICE_BASE.
    pub unsafe fn init() {
        // Activate Distributor
        mmio_write32(GICD_BASE + GICD_CTLR, 0x1);

        // Activate CPU interface
        mmio_write32(GICC_BASE + GICC_PMR, 0xFF); // max priority
        mmio_write32(GICC_BASE + GICC_CTLR, 0x1);
    }

    /// Activate a given IRQ (global number)
    pub unsafe fn enable_irq(irq: u32) {
        let reg = (irq / 32) as usize;
        let bit = irq % 32;
        let addr = GICD_BASE + GICD_ISENABLER0 + reg * 4;
        let val = 1u32 << bit;
        mmio_write32(addr, val);
    }

    /// Read current IRQ (IAR)
    pub unsafe fn ack() -> u32 {
        mmio_read32(GICC_BASE + GICC_IAR)
    }

    /// Signal end of IRQ handling
    pub unsafe fn eoi(irq: u32) {
        mmio_write32(GICC_BASE + GICC_EOIR, irq);
    }
}