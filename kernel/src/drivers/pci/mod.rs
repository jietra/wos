// src/drivers/pci/mod.rs

pub mod host;
pub mod device;
pub mod virtio_blk;

use device::{PCI_DEVICES, PciDevice};
use virtio_blk::VirtioBlkPci;

pub static mut VIRTIO_BLK: Option<VirtioBlkPci> = None;

pub unsafe fn init_pci() {
    // 1. Instantiate device
    VIRTIO_BLK = Some(VirtioBlkPci::new());

    // 2. Retrieve mutable ref to device
    let dev_ref: &mut dyn PciDevice = VIRTIO_BLK.as_mut().unwrap();
    crate::uart_println!(
        "\t[INIT PCI] virtio-blk at bus={:02x}, dev={:02x}, func={:02x}",
        dev_ref.bus(), dev_ref.dev(), dev_ref.func()
    );

    // 3. Store in PCI table
    PCI_DEVICES[5] = Some(dev_ref); // Test device 5
}
