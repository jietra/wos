// src/drivers/pci/device.rs

/// Common PCI device structure trait

pub trait PciDevice {
    fn bus(&self) -> u8;
    fn dev(&self) -> u8;
    fn func(&self) -> u8;

    fn read_config(&self, offset: u16) -> u32;
    fn write_config(&mut self, offset: u16, value: u32);

    fn bar_mmio_base(&self, bar: u8) -> Option<u64>;
    fn bar_mmio_size(&self, bar: u8) -> Option<u64>;

    fn mmio_read(&self, offset: u64, size: u8) -> u32;
    fn mmio_write(&mut self, offset: u64, size: u8, value: u64);
}

//pub static mut PCI_DEVICES: [Option<&'static mut dyn PciDevice>; 8] = [None; 8];
//pub static mut PCI_DEVICES: [Option<*mut dyn PciDevice>; 8] = [None; 8];
pub static mut PCI_DEVICES: [Option<&'static mut dyn PciDevice>; 8] = [
    None, None, None, None, None, None, None, None
];

pub fn pci_mmio_read(ipa: u64, size: u8) -> u32 {
    unsafe {
        for d in PCI_DEVICES.iter().flatten() {
            //let dev = &**d; // *mut dyn PciDevice -> &mut dyn PciDevice -> &dyn PciDevice
            if let Some(base) = d.bar_mmio_base(0) {
                let size_bar = d.bar_mmio_size(0).unwrap();
                if ipa >= base && ipa < base + size_bar {
                    let off = ipa - base;
                    return d.mmio_read(off, size);
                }
            }
        }
    }
    0
}

pub fn pci_mmio_write(ipa: u64, size: u8, value: u64) {
    unsafe {
        for d in PCI_DEVICES.iter_mut().flatten() {
            if let Some(base) = d.bar_mmio_base(0) {
                let size_bar = d.bar_mmio_size(0).unwrap();
                if ipa >= base && ipa < base + size_bar {
                    let off = ipa - base;
                    d.mmio_write(off, size, value);
                    return;
                }
            }
        }
    }
}

