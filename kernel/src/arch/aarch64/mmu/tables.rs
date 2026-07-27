// src/arch/aarch64/mmu/tables.rs

use crate::arch::aarch64::boot::linker_symbols as ls;
use crate::memory::memory_layout::layout::{DEVICE_BASE, KERNEL_BASE, KERNEL_HEAP_BASE};
use crate::memory::virt::{l0_index, l1_index, l2_index, l3_index};
use crate::memory::phys::alloc_page;

// -----------------------------------------------------------------------------
// Minimal page tables
// (4-level, 512 entries each, 4KB pages)
// -----------------------------------------------------------------------------
#[repr(align(4096))]
pub struct PageTable(pub [u64; 512]);

#[link_section = ".boot_tables"]
pub static mut L0_TABLE: PageTable = PageTable([0; 512]);   // TODO: can be deleted
#[link_section = ".boot_tables"]
pub static mut L0_LOW : PageTable = PageTable([0; 512]);
#[link_section = ".boot_tables"]
pub static mut L0_HIGH: PageTable = PageTable([0; 512]);

// Page attibute (MAIR index)
#[derive(Copy, Clone)]
pub enum PageAttr {
    Normal = 2, // attr_index=2 for Normal WB
    Device = 0, // attr_index=0 for Device-nGnRnE
}

// -----------------------------------------------------------------------------
// Minimal page tables initialization
// (maps 0x4008_0000 for kernel and 0x0900_0000 for UART)
// -----------------------------------------------------------------------------
pub unsafe fn init_page_tables() {
    crate::uart_println!("\tInitializing page tables...");

    // Empty L0
    L0_LOW.0  = [0; 512];

    // 0. Identity map minimal (just enough to survive MMU enable)
    identity_map_boot_region();

    // 1. Kernel high-half
    map_kernel_l3();

    // 2. Kernel heap high-half
    map_kernel_heap_l3();

    // 3. Devices high-half
    map_devices();

    // 4. User space (identity mapping for now)
    map_user_space();
}

// -----------------------------------------------------------------------------
// Kernel high-half mapping
// -----------------------------------------------------------------------------
pub fn kernel_start_phys() -> u64 {
    unsafe { &ls::_kernel_start as *const u8 as u64 }
}

pub unsafe fn map_kernel_l3() {
    /// Map the kernel code/data/bss into the high-half virtual address space
    crate::uart_println!("\t\tMapping kernel high-half...");

    let ks = kernel_start_phys();
    let ke = &ls::_kernel_end as *const u8 as u64;
    let ke_extended = (ke + 0x200000) & !0xFFF;
    let size = ke_extended - ks;

    let L0 = &mut L0_HIGH;

    map_region(
        L0,
        KERNEL_BASE as u64,
        ks,
        size,
        PageAttr::Normal,
        true,
        0b00
    );

    // CHECK Dump
    let exc_phys = &ls::_exceptions_start as *const u8 as u64;
    let exc_virt = KERNEL_BASE as u64 + (exc_phys - kernel_start_phys());
    crate::uart_println!("\t\t\t_exceptions_start PA = 0x{:016x}", exc_phys);
    crate::uart_println!("\t\t\t_exceptions_start VA = 0x{:016x}", exc_virt);
    map_region(
        &mut L0_HIGH,
        exc_virt,
        exc_phys,
        0x2000,             // size of the exception table
        PageAttr::Normal,
        true,
        0b00
    );

    // set VBAR_EL1 in va
    core::arch::asm!(
        "msr VBAR_EL1, {0}",
        "isb",
        in(reg) exc_virt,
    );

    let mut vbar: u64;
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    crate::uart_println!("\t\t\tVBAR_EL1 set in va   = 0x{:016x}", vbar);

    crate::uart_println!("\t\t\t--- checking translation with KERNEL_BASE before MMU ---");
    let va = KERNEL_BASE as u64;
    let i0 = l0_index(va);
    let i1 = l1_index(va);
    let i2 = l2_index(va);
    let i3 = l3_index(va);

    crate::uart_println!("\t\t\tKERNEL_BASE VA = 0x{:016x}", va);
    crate::uart_println!("\t\t\tindices: L0={}", i0);
    crate::uart_println!("\t\t\tindices: L1={}", i1);
    crate::uart_println!("\t\t\tindices: L2={}", i2);
    crate::uart_println!("\t\t\tindices: L3={}", i3);
    crate::uart_println!("\t\t\tL0_HIGH[i0] = 0x{:016x}", L0.0[i0]);

    let l1_pa = L0.0[i0] & !0xFFF;
    let l1 = l1_pa as *const u64;
    crate::uart_println!("\t\t\tL1[i1] = 0x{:016x}", unsafe { *l1.add(i1) });

    let l2_pa = unsafe { *l1.add(i1) } & !0xFFF;
    let l2 = l2_pa as *const u64;
    crate::uart_println!("\t\t\tL2[i2] = 0x{:016x}", unsafe { *l2.add(i2) });

    let l3_pa = unsafe { *l2.add(i2) } & !0xFFF;
    let l3 = l3_pa as *const u64;
    crate::uart_println!("\t\t\tL3[i3] = 0x{:016x}", unsafe { *l3.add(i3) });

    let pa = unsafe { *l3.add(i3) & !0xFFF };
    crate::uart_println!("\t\t\tKERNEL_BASE VA -> PA = 0x{:016x}", pa);

}

// -----------------------------------------------------------------------------
// Kernel Heap high-half mapping
// -----------------------------------------------------------------------------
pub unsafe fn map_kernel_heap_l3() {
    /// Map the kernel heap into the high-half virtual address space
    crate::uart_println!("\t\tMapping kernel heap high-half...");

    let hs = &ls::_heap_start as *const u8 as u64;
    let he = &ls::_heap_end   as *const u8 as u64;
    let size = he - hs;

    let L0 = &mut L0_HIGH;

    map_region(
        L0,
        KERNEL_HEAP_BASE as u64,
        hs,
        size,
        PageAttr::Normal,
        false,
        0b00
    );
}

// -----------------------------------------------------------------------------
// Device high-half mapping
// -----------------------------------------------------------------------------
pub unsafe fn map_devices() {
    /// Map the devices (UART, GIC, etc.) into the high-half virtual address space
    crate::uart_println!("\t\tMapping devices high-half...");

    let L0 = &mut L0_HIGH;

    // Map GICD
    map_region(
        L0,
        DEVICE_BASE as u64 + 0x0000_0000,
        0x0800_0000,
        0x10000,
        PageAttr::Device,
        false,
        0b00
    );
    // Map GICC
    map_region(
        L0,
        DEVICE_BASE as u64 + 0x0001_0000,
        0x0801_0000,
        0x10000,
        PageAttr::Device,
        false,
        0b00
    );
    // Map UART
    map_region(
        L0,
        DEVICE_BASE as u64 + 0x0020_0000,
        0x0900_0000,
        0x10000,
        PageAttr::Device,
        false,
        0b00
    );
}

// -----------------------------------------------------------------------------
// Boot region identity mapping
// -----------------------------------------------------------------------------
unsafe fn identity_map_boot_region() {
    /// Identity map the boot region (kernel + UART) in low-half
    crate::uart_println!("\t\tIdentity mapping boot region...");

    let ks = kernel_start_phys();
    //let ke = &ls::_stack_top as *const u8 as u64;
    let ke = &ls::_kernel_end as *const u8 as u64;
    let ke_extended = (ke + 0x200000) & !0xFFF; // +2 MiB, aligned

    crate::uart_println!("\t\t\tidentity: ks=0x{:016x}", ks);
    crate::uart_println!("\t\t\tidentity: ke=0x{:016x}", ke);

    map_region(
        &mut L0_LOW,
        ks,    // VA = PA
        ks,    // PA
        ke_extended - ks,
        PageAttr::Normal,
        true,
        0b00
    );

    // Identity map UART (for early prints)
    map_region(
        &mut L0_LOW,
        0x0900_0000, // va = pa
        0x0900_0000,
        0x10000,
        PageAttr::Device,
        false,
        0b00
    );

}

// -----------------------------------------------------------------------------
// User space mapping
// -----------------------------------------------------------------------------
pub unsafe fn map_user_space() {
    /// Map the user space sections (text, data, stack) in low-half
    crate::uart_println!("\t\tMapping user space...");
    let L0 = &mut L0_LOW; // using TTBR0 for user

    let ut_start = &ls::_user_text_start as *const u8 as u64;
    let ut_end   = &ls::_user_text_end   as *const u8 as u64;
    let ud_start = &ls::_user_data_start as *const u8 as u64;
    let ud_end   = &ls::_user_data_end   as *const u8 as u64;
    let us_start = &ls::_user_stack_start as *const u8 as u64;
    let us_end   = &ls::_user_stack_end   as *const u8 as u64;

    // texte user (RX)
    map_region(
        L0,
        ut_start,   // VA = PA for now (identity)
        ut_start,
        ut_end - ut_start,
        PageAttr::Normal,
        true,
        0b01        // AP: EL0 allowed
    );

    // data user (RW)
    map_region(
        L0,
        ud_start,
        ud_start,
        ud_end - ud_start,
        PageAttr::Normal,
        true,
        0b01
    );

    // stack user (RW)
    map_region(
        L0,
        us_start,
        us_start,
        (us_end - us_start),
        PageAttr::Normal,
        true,
        0b01
    );
}

// -----------------------------------------------------------------------------
// Map a region of memory (multiple pages) with given attributes
// -----------------------------------------------------------------------------
pub unsafe fn map_region(l0: &mut PageTable, va_start: u64, pa_start: u64, size: u64, attr: PageAttr, exec: bool, ap: u64) {
    let mut offset = 0;

    while offset < size {
        let va = va_start + offset;
        let pa = pa_start + offset;

        let i0 = l0_index(va);
        let i1 = l1_index(va);
        let i2 = l2_index(va);
        let i3 = l3_index(va);

        // --- L0 ---
        if l0.0[i0] == 0 {
            let new_l1_pa = alloc_page().expect("alloc L1 failed");
            l0.0[i0] = new_l1_pa | 0b11;
        }
        let l1 = (l0.0[i0] & !0xFFF) as *mut u64;

        // --- L1 ---
        if unsafe { *l1.add(i1) } == 0 {
            let new_l2_pa = alloc_page().expect("alloc L2 failed");
            unsafe { *l1.add(i1) = new_l2_pa | 0b11 };
        }
        let l2 = unsafe { (*l1.add(i1) & !0xFFF) as *mut u64 };

        // --- L2 ---
        if unsafe { *l2.add(i2) } == 0 {
            let new_l3_pa = alloc_page().expect("alloc L3 failed");
            unsafe { *l2.add(i2) = new_l3_pa | 0b11 };
        }
        let l3 = unsafe { (*l2.add(i2) & !0xFFF) as *mut u64 };

        // --- L3 ---
        unsafe {
            //*l3.add(i3) = l3_page_entry(pa, attr as u64, false, 0b00);
            *l3.add(i3) = l3_page_entry(pa, attr as u64, exec, ap);
        }

        offset += 0x1000;
    }
}

pub fn l3_page_entry(phys: u64, attr: u64, exec: bool, ap: u64) -> u64 {
    let mut desc =
        (phys & !((1u64 << 12) - 1)) |      // align 4 KiB
        (attr << 2) |
        (1 << 10)   |                       // AF
        (3 << 8)    |                       // SH = Inner Shareable
        (ap << 6)   |                       // AP bits
        0b11;                               // VALID + PAGE

    if !exec {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}