// src/arch/aarch64/gic/gicv2_virt.rs

pub struct VirtGicd {
    pub ctlr:      u32,
    pub isenabler: [u32; 32],   // 1024 IRQs max
    pub icenabler: [u32; 32],
    pub ipriority: [u8; 1024],
}

pub struct VirtGicc {
    pub ctlr: u32,
    pub pmr:  u32,
    pub iar:  u32,
    pub eoir: u32,
}

pub static mut VIRT_GICD: VirtGicd = VirtGicd {
    ctlr:      0,
    isenabler: [0; 32],
    icenabler: [0; 32],
    ipriority: [0; 1024],
};

pub static mut VIRT_GICC: VirtGicc = VirtGicc {
    ctlr: 0,
    pmr:  0xFF,
    iar:  0,
    eoir: 0,
};
