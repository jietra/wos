// src/drivers/pci/virtio_blk.rs

use crate::drivers::pci::device::PciDevice;
use crate::config::VIRTIO_MMIO_BASE;

const VIRTIO_VENDOR_ID: u16 = 0x1af4;
const VIRTIO_BLK_DEVICE_ID: u16 = 0x1001;

// Feature bits (on simplifie)
const VIRTIO_BLK_F_SIZE_MAX: u64    = 1 << 1;
const VIRTIO_BLK_F_SEG_MAX: u64     = 1 << 2;
const VIRTIO_BLK_F_GEOMETRY: u64    = 1 << 4;
const VIRTIO_BLK_F_RO: u64          = 1 << 5;
const VIRTIO_BLK_F_BLK_SIZE: u64    = 1 << 6;
const VIRTIO_F_VERSION_1: u64       = 1 << 32;

#[derive(Clone, Copy)]
struct VirtioCommonCfg {
    // offsets 0x00..0x3c (virtio 1.0 MMIO common config)
    pub device_feature_select: u32, // 0x00
    pub device_feature:        u32, // 0x04
    pub driver_feature_select: u32, // 0x08
    pub driver_feature:        u32, // 0x0C
    pub msix_config:           u16, // 0x10
    pub num_queues:            u16, // 0x12
    pub device_status:         u8,  // 0x14
    pub config_generation:     u8,  // 0x15

    pub queue_select:          u16, // 0x16
    pub queue_size:            u16, // 0x18
    pub queue_msix_vector:     u16, // 0x1A
    pub queue_enable:          u16, // 0x1C
    pub queue_notify_off:      u16, // 0x1E

    pub queue_desc:            u64, // 0x20
    pub queue_driver:          u64, // 0x28
    pub queue_device:          u64, // 0x30
}

impl VirtioCommonCfg {
    fn new() -> Self {
        Self {
            device_feature_select: 0,
            device_feature:        0, // we will set VIRTIO_BLK_F_* later
            driver_feature_select: 0,
            driver_feature:        0,
            msix_config:           0,
            num_queues:            1,
            device_status:         0,
            config_generation:     0,
            queue_select:          0,
            queue_size:            128, // max queue size
            queue_msix_vector:     0,
            queue_enable:          0,
            queue_notify_off:      0,
            queue_desc:            0,
            queue_driver:          0,
            queue_device:          0,
        }
    }

    fn device_features(&self, sel: u32) -> u32 {
        let all = VIRTIO_BLK_F_SIZE_MAX
            | VIRTIO_BLK_F_SEG_MAX
            | VIRTIO_BLK_F_GEOMETRY
            | VIRTIO_BLK_F_BLK_SIZE
            | VIRTIO_F_VERSION_1;
        let shift = (sel as u64) * 32;
        ((all >> shift) & 0xffff_ffff) as u32
    }
}

#[derive(Clone, Copy)]
struct VirtioQueue {
    size:   u16,
    desc:   u64,
    driver: u64,
    device: u64,
    enabled: bool,
}

impl VirtioQueue {
    fn new() -> Self {
        Self {
            size:   0,
            desc:   0,
            driver: 0,
            device: 0,
            enabled: false,
        }
    }
}

pub struct VirtioBlkPci {
    bus:  u8,
    dev:  u8,
    func: u8,
    bar0: u32, // 0x7000_0000

    common_cfg: VirtioCommonCfg,
    queues:     [VirtioQueue; 1],
    isr_status: u8,
}

impl VirtioBlkPci {
    pub fn new() -> Self {
        Self {
            bus:  0,
            dev:  5,
            func: 0,
            bar0: VIRTIO_MMIO_BASE as u32,
            common_cfg: VirtioCommonCfg::new(),
            queues:     [VirtioQueue::new()],
            isr_status: 0,
        }
    }

    fn mmio_read_common(&self, offset: u64) -> u32 {
        match offset {
            0x00 => self.common_cfg.device_feature_select,
            0x04 => self.common_cfg.device_features(self.common_cfg.device_feature_select),
            0x08 => self.common_cfg.driver_feature_select,
            0x0c => self.common_cfg.driver_feature,
            0x10 => self.common_cfg.msix_config as u32,
            0x12 => self.common_cfg.num_queues as u32,
            0x14 => self.common_cfg.device_status as u32,
            0x15 => self.common_cfg.config_generation as u32,
            0x16 => self.common_cfg.queue_select as u32,
            0x18 => self.common_cfg.queue_size as u32,
            0x1a => self.common_cfg.queue_msix_vector as u32,
            0x1c => self.common_cfg.queue_enable as u32,
            0x1e => self.common_cfg.queue_notify_off as u32,
            0x20 => (self.common_cfg.queue_desc & 0xffff_ffff) as u32,
            0x24 => (self.common_cfg.queue_desc >> 32) as u32,
            0x28 => (self.common_cfg.queue_driver & 0xffff_ffff) as u32,
            0x2c => (self.common_cfg.queue_driver >> 32) as u32,
            0x30 => (self.common_cfg.queue_device & 0xffff_ffff) as u32,
            0x34 => (self.common_cfg.queue_device >> 32) as u32,
            _    => 0,
        }
    }

    fn mmio_write_common(&mut self, offset: u64, value: u32) {
        match offset {
            0x00 => self.common_cfg.device_feature_select = value,
            0x08 => self.common_cfg.driver_feature_select = value,
            0x0C => self.common_cfg.driver_feature        = value,
            0x10 => self.common_cfg.msix_config           = value as u16,
            0x12 => self.common_cfg.num_queues            = value as u16,
            0x14 => self.common_cfg.device_status         = value as u8,
            0x15 => self.common_cfg.config_generation     = value as u8,
            0x16 => self.common_cfg.queue_select          = value as u16,
            0x18 => {
                self.common_cfg.queue_size = value as u16;
                let q = &mut self.queues[self.common_cfg.queue_select as usize];
                q.size = self.common_cfg.queue_size;
            }
            0x1A => self.common_cfg.queue_msix_vector     = value as u16,
            0x1C => {
                self.common_cfg.queue_enable = value as u16;
                let q = &mut self.queues[self.common_cfg.queue_select as usize];
                q.enabled = self.common_cfg.queue_enable != 0;
            }
            0x1E => self.common_cfg.queue_notify_off      = value as u16,
            0x20 | 0x24 => {
                let mut v = self.common_cfg.queue_desc;
                if offset == 0x20 {
                    v = (v & !0xffff_ffff) | (value as u64);
                } else {
                    v = (v & 0xffff_ffff) | ((value as u64) << 32);
                }
                self.common_cfg.queue_desc = v;
                let q = &mut self.queues[self.common_cfg.queue_select as usize];
                q.desc = v;
            }
            0x28 | 0x2C => { /* queue_driver */ }
            0x30 | 0x34 => { /* queue_device */ }
            _ => {}
        }
    }

    fn mmio_read_isr(&self, _offset: u64) -> u32 {
        self.isr_status as u32
    }

    fn mmio_write_isr(&mut self, _offset: u64, _value: u32) {
        // ack interrupt
        self.isr_status = 0;
    }
}

impl PciDevice for VirtioBlkPci {
    fn bus(&self) -> u8  { self.bus }
    fn dev(&self) -> u8  { self.dev }
    fn func(&self) -> u8 { self.func }

    fn read_config(&self, offset: u16) -> u32 {
        match offset {
            // 0x00–0x03 : Vendor ID + Device ID
            0x00 => {
                (0x1af4u32) | ((0x1001u32) << 16)
            }

            // 0x08–0x0B : Class code, subclass, progIF, revision
            // class = 0x01 (mass storage), subclass/progif/rev = 0
            0x08 => {
                (0x01u32 << 24) // class
            }

            // 0x0C–0x0F : header type, etc.
            // header type = 0 (type 0), reste 0
            0x0C => {
                0x0000_0000
            }

            // 0x10–0x13 : BAR0
            0x10 => {
                self.bar0
            }


            _ => 0,
        }
    }

    fn write_config(&mut self, offset: u16, value: u32) {
        if offset == 0x10 {
            self.bar0 = value & !0xFFF; // aligned
        }
    }

    fn bar_mmio_base(&self, bar: u8) -> Option<u64> {
        if bar == 0 { Some(self.bar0 as u64) } else { None }
    }

    fn bar_mmio_size(&self, bar: u8) -> Option<u64> {
        if bar == 0 { Some(0x1000) } else { None }
    }

    fn mmio_read(&self, offset: u64, size: u8) -> u32 {
        // common config at BAR0 base
        if offset < 0x40 {
            let val32 = self.mmio_read_common(offset);
            match size {
                1 => (val32 & 0xFF) as u32,
                2 => (val32 & 0xFFFF) as u32,
                4 | 8 => val32 as u32,
                _ => val32 as u32,
            }
        } else if offset == 0x100 { // ISR status (exemple)
            self.mmio_read_isr(offset) as u32
        } else {
            0
        }
    }

    fn mmio_write(&mut self, offset: u64, _size: u8, value: u64) {
        let v32 = value as u32;
        if offset < 0x40 {
            self.mmio_write_common(offset, v32);
        } else if offset == 0x100 {
            self.mmio_write_isr(offset, v32);
        } else {
            // notify, etc.
        }
    }
}
