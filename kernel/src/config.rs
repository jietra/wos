// src/config.rs

/// Configuration file (static layout)

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::mmu::stage2::{S2Table, Perm};

pub const GUEST1_PA_BASE:  u64 = 0x4300_0000;
pub const GUEST1_IPA_BASE: u64 = 0x4000_0000;
pub const GUEST1_IPA_SIZE: u64 = 0x3000_0000;   // has to be less than 1GiB-(GUEST1_PA_BASE-GUEST1_IPA_BASE)=0x4000_0000-(0x4300_0000-0x4000_0000)=0x3D00_0000
                                                // adjust dts

pub const TEXT_OFFSET:     u64 = 0x80000;       // as per documentation (offset 0x0008_0000)
pub const LINUX_ENTRY_IPA: u64 = align_up(GUEST1_IPA_BASE + TEXT_OFFSET, 0x0020_0000);//=0x4320_0000; Linux does not like misaligned kernels

pub const GUEST_UART_IPA:  u64 = 0x0900_0000;   // allow outputs on hypervisor uart
pub const GUEST_GICD_IPA:  u64 = 0x08000000;
pub const GUEST_GICC_IPA:  u64 = 0x08010000;
pub const GUEST_VIRTIO_BLK_IPA:  u64 = 0x0A00_0000;

pub const VIRTIO_MMIO_BASE: u64 = 0x0a000000;   //NB PCI BAR0 @0x7000_0000;

// static VM struct
pub struct StaticVmConfig {
    pub id: u8,
    pub name: &'static str,
    pub ipa_range: (u64, u64),
    //pub s2_root: *const S2Table,
    pub image_kernel: &'static [u8],
    pub image_initramfs: Option<&'static [u8]>,
    pub dtb: &'static [u8],
    pub console_mmio_base: u64,
}

/// Table of VMs configs
pub static VM_CONFIGS: &[StaticVmConfig] = &[
    StaticVmConfig {
        id: 0,
        name: "kwalt",
        ipa_range: (0x0000_0000, 0x3FFF_FFFF),
        //s2_root: &S2_PT_GUEST0,
        image_kernel: GUEST0_ELF,
        image_initramfs: None,
        dtb: &[],
        console_mmio_base: 0x1000_0000,
    },
    StaticVmConfig {
        id: 1,
        name: "linux_guest",
        ipa_range: (GUEST1_IPA_BASE, GUEST1_IPA_BASE + GUEST1_IPA_SIZE - 1),
        //s2_root: &S2_PT_GUEST1,
        image_kernel: GUEST1_KERNEL,
        image_initramfs: Some(&GUEST1_INITRAMFS_DATA),
        dtb: GUEST1_DTB,
        console_mmio_base: GUEST_UART_IPA,//0x1001_0000,
    },
];

/// Number of VMs
pub const NUM_VMS: usize = VM_CONFIGS.len();

/// Guests identifiers
pub const VM_KWALT: usize = 0;  // guest 0: critical: root OS: supervise, filter, expose userland services, manage resources, provide secure services to other guests
pub const VM_LINUX: usize = 1;  // guest 1: non critical: general purpose (e.g. AI workloads, compute, perception...): zero-trust, sandboxed

/// IPA memory size for each guest
pub const GUEST0_IPA_SIZE: u64 = 0x4000_0000;

/// Entry addresses
pub const KWALT_ENTRY_IPA: u64 = 0x0000_8000;

/*
// TODO: for future use with multiple VMs -> dedicated S2 tables...
// src/arch/aarch64/mmu/stage2.rs
pub const fn build_guest0_s2() -> S2Table {
    let mut pt = S2Table::new();

    // code RX
    pt.map_range(0x0000_0000, 0x0000_8000, 0x8000_0000, Perm::RX);

    // data RW
    pt.map_range(0x0000_8000, 0x0003_FFFF, 0x8000_8000, Perm::RW);

    // stack RW
    pt.map_range(0x0004_0000, 0x0004_FFFF, 0x8004_0000, Perm::RW);

    pt
}

pub const fn build_guest1_s2() -> S2Table {
    let mut pt = S2Table::new();

    // RAM guest Linux (identity mapping IPA->PA)
    pt.map_range(0x4000_0000, 0x7FFF_FFFF, 0x4000_0000, Perm::RW);

    // GICv2
    pt.map_device(0x0800_0000, 0x0800_0000); // GICD
    pt.map_device(0x0801_0000, 0x0801_0000); // GICC

    // UART PL011
    pt.map_device(0x0900_0000, 0x0900_0000);

    // virtio-blk
    pt.map_device(0x0A00_0000, 0x0A00_0000);

    pt
}

pub static S2_PT_GUEST0: S2Table = build_guest0_s2();
pub static S2_PT_GUEST1: S2Table = build_guest1_s2();
*/

// TODO: we may want to move that part to some src/arch/aarch64/images.rs
pub const GUEST0_ELF_SIZE:          usize = include_bytes!("../../images/kwaltARM64.elf").len();
pub const GUEST1_KERNEL_SIZE:       usize = include_bytes!("../../images/linux/Image").len();
pub const GUEST1_DTB_SIZE:          usize = include_bytes!("../../images/linux/guest1.dtb").len();
pub const GUEST1_INITRAMFS_SIZE:    usize = include_bytes!("../../images/linux/rootfs.cpio").len();

//#[link_section = ".guest_images"] // directly using include_bytes! actually copies the binary to the rodata section, leaving the guest_images section almost empty and worthless
pub static GUEST0_ELF: &[u8]                    = include_bytes!("../../images/kwaltARM64.elf");
//pub static GUEST0_ELF: [u8; GUEST0_ELF_SIZE]    = *include_bytes!("../../images/kwaltARM64.elf");
// TODO: using *include_bytes! would ease the rodata section and actually move the binary to the guest_images section: however, the blob_copy won't work in this case...

//#[link_section = ".guest_images"]
pub static GUEST1_KERNEL: &[u8]                     = include_bytes!("../../images/linux/Image");
//pub static GUEST1_KERNEL: [u8; GUEST1_KERNEL_SIZE]  = *include_bytes!("../../images/linux/Image");

//#[link_section = ".guest_images"]
pub static GUEST1_DTB: &[u8]                    = include_bytes!("../../images/linux/guest1.dtb");
//pub static GUEST1_DTB: [u8; GUEST1_DTB_SIZE]    = *include_bytes!("../../images/linux/guest1.dtb");

//#[link_section = ".guest_images"]
pub static GUEST1_INITRAMFS_DATA: [u8; GUEST1_INITRAMFS_SIZE] = *include_bytes!("../../images/linux/rootfs.cpio");
pub static GUEST1_INITRAMFS: Option<&'static [u8]> = Some(&GUEST1_INITRAMFS_DATA);

//#[link_section = ".guest_images"]
//pub static GUEST1_INITRAMFS: Option<&'static [u8]> = Some(include_bytes!("../../images/linux/rootfs.cpio.gz"));

// When no initramfs:
//pub static GUEST1_INITRAMFS: Option<&'static [u8]> = None;
// we could also use a test like:
//if let Some(initramfs) = GUEST1_INITRAMFS {
//    load_initramfs(initramfs);
//}

// helper function (aligning entry)
#[inline]
const fn align_up(x: u64, align: u64) -> u64 {
    (x + (align - 1)) & !(align - 1)
}