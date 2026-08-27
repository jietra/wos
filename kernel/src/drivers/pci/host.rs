// src/drivers/pci/host.rs

/// Manage PCI bus, ECAM, dispatch

const ECAM_BASE: u64 = 0x1000_0000;

use crate::drivers::pci::device::PCI_DEVICES;

/*
pub fn ecam_read(ipa: u64) -> u32 {
    let off = ipa - ECAM_BASE;

    let bus  = ((off >> 20) & 0xFF) as u8;
    let dev  = ((off >> 15) & 0x1F) as u8;
    let func = ((off >> 12) & 0x07) as u8;
    let reg  = (off & 0xFFF) as u16;

    crate::uart_println!("[ECAM] read bus={}, dev={}, func={}, reg=0x{:03x}", bus, dev, func, reg);

    unsafe {
        for d in PCI_DEVICES.iter().flatten() {
            //let dd = &mut **d;
            if d.bus() == bus && d.dev() == dev && d.func() == func {
                return d.read_config(reg);
            }
        }
    }

    0xFFFF_FFFF // no device
}
*/
/*
pub fn ecam_read(ipa: u64, size: u8) -> u64 {
    let off = ipa - ECAM_BASE;
    let bus  = ((off >> 20) & 0xFF) as u8;
    let dev  = ((off >> 15) & 0x1F) as u8;
    let func = ((off >> 12) & 0x07) as u8;
    let reg  = (off & 0xFFF) as u16;

    let val32 = unsafe {
        PCI_DEVICES
            .iter()
            .flatten()
            .find(|d| d.bus() == bus && d.dev() == dev && d.func() == func)
            .map(|d| d.read_config(reg))
            .unwrap_or(0xffff_ffff)
    };

    match size {
        1 => (val32 & 0xFF) as u64,
        2 => (val32 & 0xFFFF) as u64,
        4 => val32 as u64,
        _ => 0xffff_ffff,
    }
}
*/
pub fn ecam_read(ipa: u64, size: u8) -> u32 {
    let off = ipa - ECAM_BASE;
    let bus  = ((off >> 20) & 0xFF) as u8;
    let dev  = ((off >> 15) & 0x1F) as u8;
    let func = ((off >> 12) & 0x07) as u8;
    let reg  = (off & 0xFFF) as u16;

    // aligner sur 4 octets
    let reg_aligned = reg & !0x3;
    let byte_offset = (reg & 0x3) as u32;

    let val32 = unsafe {
        PCI_DEVICES
            .iter()
            .flatten()
            .find(|d| d.bus() == bus && d.dev() == dev && d.func() == func)
            .map(|d| d.read_config(reg_aligned))
            .unwrap_or(0xffff_ffff)
    };

    let ret = match size {
        1 => {
            let shift = byte_offset * 8;
            ((val32 >> shift) & 0xFF) as u32
        }
        2 => {
            let shift = (byte_offset & !0x1) * 8; // aligned on 2 octets
            ((val32 >> shift) & 0xFFFF) as u32
        }
        4 => val32,
        _ => 0xffff_ffff,
    };

    crate::uart_println!("[ECAM] read dev={:02x}, reg=0x{:03x}, reg_aligned=0x{:03x}, return=0x{:08x}", dev, reg, reg_aligned, ret);
    ret
}


pub fn ecam_write(ipa: u64, value: u32) {
    let off = ipa - ECAM_BASE;

    let bus  = ((off >> 20) & 0xFF) as u8;
    let dev  = ((off >> 15) & 0x1F) as u8;
    let func = ((off >> 12) & 0x07) as u8;
    let reg  = (off & 0xFFF) as u16;

    unsafe {
        for d in PCI_DEVICES.iter_mut().flatten() {
            //let dd = &mut **d;
            if d.bus() == bus && d.dev() == dev && d.func() == func {
                d.write_config(reg, value);
                return;
            }
        }
    }
}
