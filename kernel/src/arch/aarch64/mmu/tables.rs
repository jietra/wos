// src/arch/aarch64/mmu/tables.rs

use crate::arch::aarch64::boot::linker_symbols as ls;
use crate::memory::memory_layout::layout::{DEVICE_BASE, KERNEL_BASE};

/*
/// Fallback in case of pb with exception vector address: Save _exceptions_start before MMU
pub static mut EXC_PA: u64 = 0;
pub fn save_exc_pa() {
    unsafe { EXC_PA = &ls::_exceptions_start as *const u8 as u64; }
}
*/

// -----------------------------------------------------------------------------
// For high half kernel
// -----------------------------------------------------------------------------
#[repr(align(4096))]
pub struct Table([u64; 512]);

#[no_mangle]
pub static mut L1_KERNEL_HI: Table = Table([0; 512]);
#[no_mangle]
pub static mut L2_KERNEL_HI: Table = Table([0; 512]);
#[no_mangle]
pub static mut L3_KERNEL_HI: Table = Table([0; 512]);


// -----------------------------------------------------------------------------
// Minimal page tables
// (4-level, 512 entries each, 4KB pages)
// -----------------------------------------------------------------------------
#[repr(align(4096))]
pub struct PageTable(pub [u64; 512]);

pub static mut L0_TABLE: PageTable = PageTable([0; 512]);
pub static mut L1_TABLE: PageTable = PageTable([0; 512]);
pub static mut L2_TABLE: PageTable = PageTable([0; 512]);
pub static mut L3_KERNEL_TABLE: PageTable = PageTable([0; 512]);

pub static mut L1_DEVICE_TABLE: PageTable = PageTable([0; 512]);
pub static mut L2_DEVICE_TABLE: PageTable = PageTable([0; 512]);

// Page attibute (MAIR index)
pub enum PageAttr {
    Normal = 2, // attr_index=2 for Normal WB
    Device = 0, // attr_index=0 for Device-nGnRnE
}

// -----------------------------------------------------------------------------
// Descriptors
// -----------------------------------------------------------------------------
// Helper function to create a block entry in the page table
pub fn l1_block_entry(phys: u64, attr: u64, exec: bool) -> u64 {
    let mut desc = (phys & !((1 << 30) - 1))    // Align physical address to 1 GB // use 0x0000_FFFF_FFFF_0000 or !((1<<12)-1) for 4KB pages alignment (L3), !((1<<21)-1) for 2MB blocks (L2)
        | (attr << 2)
        | (1 << 10)                             // AF
        | (3 << 8)                              // SH = Inner Shareable
        | (0 << 6)                              // AP = RW EL1
        | 0b01;                                 // VALID + BLOCK

    if !exec {
        // PXN + UXN = Execute Never for both privileged and unprivileged
        desc |= (1 << 54) | (1 << 53);
    }

    desc
}

pub fn l3_page_entry(phys: u64, attr_index: u64, executable: bool, ap: u64) -> u64 {
    let mut desc =
        (phys & !((1u64 << 12) - 1)) |      // align 4 KiB
        (attr_index << 2) |
        (1 << 10) |                         // AF
        (3 << 8)  |                         // SH = Inner Shareable
        (ap << 6)  |                        // AP bits
        0b11;                               // VALID + PAGE

    if !executable {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}

// Map a 4K page (L3)
pub unsafe fn map_page(virt: u64, phys: u64, attr: PageAttr) {
    // Assumes virt is in DEVICE_BASE..DEVICE_BASE+1TB
    // Take the entry L0[511] to L1_DEVICE_TABLE
    // then L1_DEVICE[0] to L2_DEVICE_TABLE
    // then computes L3 index.

    //let l0_index = ((virt >> 39) & 0x1FF) as usize;
    //let l1_index = ((virt >> 30) & 0x1FF) as usize;
    let l2_index = ((virt >> 21) & 0x1FF) as usize;
    let l3_index = ((virt >> 12) & 0x1FF) as usize;

    // L0[511] already pointing to L1_DEVICE_TABLE
    // L1_DEVICE[0] already pointing to L2_DEVICE_TABLE

    // Assumes a unique L3 for this minimal config:
    static mut L3_DEVICE_TABLE: PageTable = PageTable([0; 512]);

    // L2_DEVICE_TABLE[l2_index] -> L3_DEVICE_TABLE
    if L2_DEVICE_TABLE.0[l2_index] == 0 {
        L2_DEVICE_TABLE.0[l2_index] =
            (&raw const L3_DEVICE_TABLE as *const _ as u64) | 0b11;
    }

    let attr_index = attr as u64;
    let ap = 0b00; // RW EL1
    let executable = false;

    L3_DEVICE_TABLE.0[l3_index] = l3_page_entry(phys, attr_index, executable, ap);
}

// -----------------------------------------------------------------------------
// Kernel mapping
// -----------------------------------------------------------------------------

pub fn kernel_start_phys() -> u64 {
    unsafe { &ls::_kernel_start as *const u8 as u64 }
}

pub fn phys_to_kernel_virt(pa: u64) -> u64 {
    let ks = kernel_start_phys();
    // temporary: avoid underflow
    if pa < ks {
        // log / println / return pa (debug)
        return pa;
    }
    KERNEL_BASE as u64 + (pa - ks)
}

pub unsafe fn map_kernel_l3() {
    let kernel_start = &ls::_kernel_start as *const u8 as u64;
    let kernel_end   = &ls::_stack_top    as *const u8 as u64;

    let text_start   = &ls::_text_start   as *const u8 as u64;
    let text_end     = &ls::_text_end     as *const u8 as u64;
    let exc_start    = &ls::_exceptions_start as *const u8 as u64;
    let exc_end      = &ls::_exceptions_end   as *const u8 as u64;
    let rodata_start = &ls::_rodata_start as *const u8 as u64;
    let rodata_end   = &ls::_rodata_end   as *const u8 as u64;

    let mut phys = kernel_start;
    while phys < kernel_end {
        let va = phys_to_kernel_virt(phys);

        let (attr_index, ap, exec) =
            if phys >= text_start && phys < text_end {
                (2, 0b10, true)
            } else if phys >= exc_start && phys < exc_end {
                (2, 0b10, true)
            } else if phys >= rodata_start && phys < rodata_end {
                (2, 0b10, false)
            } else {
                (2, 0b00, false)
            };

        let l3_index = ((va >> 12) & 0x1FF) as usize;
        L3_KERNEL_HI.0[l3_index] = l3_page_entry(phys, attr_index, exec, ap);

        phys += 0x1000;
    }

    let va_start = phys_to_kernel_virt(kernel_start);
    let va_end   = phys_to_kernel_virt(kernel_end - 1);

    let l2_index = ((va_start >> 21) & 0x1FF) as usize;
    L2_KERNEL_HI.0[l2_index] = (&raw const L3_KERNEL_HI as *const _ as u64) | 0b11;
}

// -----------------------------------------------------------------------------
// Minimal page tables initialization
// (maps 0x4008_0000 for kernel and 0x0900_0000 for UART)
// -----------------------------------------------------------------------------
pub unsafe fn init_page_tables() {
    // Kernel low-half
    L0_TABLE.0[0] = (&raw const L1_TABLE as *const _ as u64) | 0b11;    // L0[0] -> L1
    L1_TABLE.0[0] = l1_block_entry(0, 0, false);                        // L1[0]: 0–1 Go: Device
    L1_TABLE.0[1] = (&raw const L2_TABLE as *const _ as u64) | 0b11;    // L1[1]: 1–2 Go -> KERNEL

    // High-half devices: L0[511] → L1_DEVICE
    //L0_TABLE.0[511] = (&raw const L1_DEVICE_TABLE as *const _ as u64) | 0b11;
    let dev_l0_index = ((DEVICE_BASE as u64 >> 39) & 0x1FF) as usize;   // = 0x1FA
    L0_TABLE.0[dev_l0_index] = (&raw const L1_DEVICE_TABLE as *const _ as u64) | 0b11;

    L1_DEVICE_TABLE.0[0] = (&raw const L2_DEVICE_TABLE as *const _ as u64) | 0b11;

    let k_l0_index = ((KERNEL_BASE as u64 >> 39) & 0x1FF) as usize;
    unsafe {
        L0_TABLE.0[k_l0_index] = (&raw const L1_KERNEL_HI as *const _ as u64) | 0b11;
        L1_KERNEL_HI.0[0]      = (&raw const L2_KERNEL_HI as *const _ as u64) | 0b11;
    }

    /// GIC mapping: for now we keep both identity & high half mapping
    // High-half mapping
    // Map GICD
    map_device(
        0x0800_0000,
        DEVICE_BASE as u64 + 0x0000_0000,
        0x10000,
    );
    // Map GICC
    map_device(
        0x0801_0000,
        DEVICE_BASE as u64 + 0x0001_0000,
        0x10000,
    );
    
    // identity map gicd/gicc
    map_identity_page(0x0800_0000); // GICD
    map_identity_page(0x0801_0000); // GICC

    /// UART mapping: fully in high half
    // Identity mapping for UART (BEFORE MMU)
    //map_identity_page(0x0900_0000);       // no longer needed since uart va activated in mmu init

    // High-half mapping for UART (AFTER MMU)
    map_device(
        0x0900_0000,                        // phys
        DEVICE_BASE as u64 + 0x0020_0000,   // virt
        0x10000,
    );

    /// KERNEL: for now we keep both identity mapping & high half
    // (identiy mapping of the kernel is indeed necessary during MMU activation)
    // Identity map of kernel in low-half
    let kernel_start = kernel_start_phys();
    let kernel_end   = &ls::_stack_top as *const u8 as u64;

    let mut phys = kernel_start;
    while phys < kernel_end {
        map_identity_page_kernel(phys);
        phys += 0x1000;
    }

    map_kernel_l3();

    crate::uart_println!("\t\tDEVICE BASE = ", DEVICE_BASE as u64);
    
    let va = DEVICE_BASE as u64;
    let l0 = ((va >> 39) & 0x1FF) as usize;
    let l1 = ((va >> 30) & 0x1FF) as usize;
    let l2 = ((va >> 21) & 0x1FF) as usize;
    let l3 = ((va >> 12) & 0x1FF) as usize;

    crate::uart_println!("\t\tDEVICE_BASE indices: L0={}", l0);
    crate::uart_println!("\t\tDEVICE_BASE indices: L1={}", l1);
    crate::uart_println!("\t\tDEVICE_BASE indices: L2={}", l2);
    crate::uart_println!("\t\tDEVICE_BASE indices: L3={}", l3);
    crate::uart_println!("\t\tL0[l0]       = 0x{:016x}", L0_TABLE.0[l0]);
    crate::uart_println!("\t\tL1_DEVICE[0] = 0x{:016x}", L1_DEVICE_TABLE.0[0]);
    crate::uart_println!("\t\tL2_DEVICE[0] = 0x{:016x}", L2_DEVICE_TABLE.0[0]);
}

pub unsafe fn map_device(phys: u64, virt: u64, size: u64) {
    let mut offset = 0;

    while offset < size {
        map_page(
            virt + offset,
            phys + offset,
            PageAttr::Device, // Device-nGnRnE
        );
        offset += 0x1000;
    }
}

pub unsafe fn map_identity_page(phys: u64) {
    let va = phys;

    let l0_index = ((va >> 39) & 0x1FF) as usize;
    let l1_index = ((va >> 30) & 0x1FF) as usize;
    let l2_index = ((va >> 21) & 0x1FF) as usize;
    let l3_index = ((va >> 12) & 0x1FF) as usize;

    if L0_TABLE.0[l0_index] == 0 {
        L0_TABLE.0[l0_index] = (&raw const L1_TABLE as *const _ as u64) | 0b11;
    }
    if L1_TABLE.0[l1_index] == 0 {
        L1_TABLE.0[l1_index] = (&raw const L2_TABLE as *const _ as u64) | 0b11;
    }
    if L2_TABLE.0[l2_index] == 0 {
        L2_TABLE.0[l2_index] = (&raw const L3_KERNEL_TABLE as *const _ as u64) | 0b11;
    }

    L3_KERNEL_TABLE.0[l3_index] =
        l3_page_entry(phys, PageAttr::Device as u64, false, 0b00);
}

pub unsafe fn map_identity_page_kernel(phys: u64) {
    let va = phys;

    let l0_index = ((va >> 39) & 0x1FF) as usize;
    let l1_index = ((va >> 30) & 0x1FF) as usize;
    let l2_index = ((va >> 21) & 0x1FF) as usize;
    let l3_index = ((va >> 12) & 0x1FF) as usize;

    if L0_TABLE.0[l0_index] == 0 {
        L0_TABLE.0[l0_index] = (&raw const L1_TABLE as *const _ as u64) | 0b11;
    }
    if L1_TABLE.0[l1_index] == 0 {
        L1_TABLE.0[l1_index] = (&raw const L2_TABLE as *const _ as u64) | 0b11;
    }
    if L2_TABLE.0[l2_index] == 0 {
        L2_TABLE.0[l2_index] = (&raw const L3_KERNEL_TABLE as *const _ as u64) | 0b11;
    }

    // Normal WB, RW, executable
    L3_KERNEL_TABLE.0[l3_index] =
        l3_page_entry(phys, PageAttr::Normal as u64, true, 0b00);
}