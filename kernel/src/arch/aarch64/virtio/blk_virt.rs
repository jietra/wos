// src/arch/aarch64/virtio/blk_virt.rs

pub struct VirtioBlk {
    pub status:          u32,
    pub device_features: u32,
    pub driver_features: u32,
    pub queue_sel:       u32,
    pub queue_num:       u32,
    pub queue_ready:     u32,
    pub queue_notify:    u32,
    pub isr_status:      u32,
    pub queue_desc_low:  u32,
    pub queue_desc_high: u32,
    pub queue_avail_low: u32,
    pub queue_avail_high:u32,
    pub queue_used_low:  u32,
    pub queue_used_high: u32,
}

pub static mut VIRTIO_BLK: VirtioBlk = VirtioBlk {
    status:          0,
    device_features: 0,
    driver_features: 0,
    queue_sel:       0,
    queue_num:       0,
    queue_ready:     0,
    queue_notify:    0,
    isr_status:      0,
    queue_desc_low:  0,
    queue_desc_high: 0,
    queue_avail_low: 0,
    queue_avail_high:0,
    queue_used_low:  0,
    queue_used_high: 0,
};
