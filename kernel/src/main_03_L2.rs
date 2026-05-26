#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Assembly entry point (_start)
// -----------------------------------------------------------------------------
global_asm!(
    r#"
    .section .text._start, "ax"
    .global _start
_start:
    // Set up a simple stack
    ldr x0, =0x40080000
    mov sp, x0

    // Jump into Rust
    bl rust_main

1:  b 1b   // Halt: infinite loop
"#
);

// Let the linker know about the exception vectors (required for interrupts, etc.)
global_asm!(include_str!("../boot/exception_vectors.S"));

// -----------------------------------------------------------------------------
// Exception vector initialization (sets VBAR_EL1 to point to our exception vectors)
// -----------------------------------------------------------------------------
// Declare the exception vectors symbol (defined in assembly) so Rust can reference it if needed
#[allow(dead_code)]
extern "C" {
    static exception_vectors: u8;
}

// -----------------------------------------------------------------------------
// Minimal C runtime shims required by libcore (symbols defined in the linker script)
// -----------------------------------------------------------------------------

extern "C" {
    static _text_start: u8;
    static _text_end: u8;
    static _rodata_start: u8;
    static _rodata_end: u8;
    static _data_start: u8;
    static _data_end: u8;
    static _bss_start: u8;
    static _bss_end: u8;
    static _kernel_start: u8;
    static _kernel_end: u8;
}

// -----------------------------------------------------------------------------
// Minimal page tables (4-level, 512 entries each, 4KB pages)
// -----------------------------------------------------------------------------
#[repr(align(4096))]
struct PageTable([u64; 512]);

static mut L0_TABLE: PageTable = PageTable([0; 512]);
static mut L1_TABLE: PageTable = PageTable([0; 512]);
static mut L2_KERNEL_TABLE: PageTable = PageTable([0; 512]);
//static mut L3_KERNEL_TABLE: PageTable = PageTable([0; 512]);

// -----------------------------------------------------------------------------
// Rust entry point
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn rust_main() {
    // Initialize exception vectors
    // Initialize MAIR (Memory Attribute Indirection Register) to set up memory attributes
    unsafe {
        init_exceptions();
        init_mair();
        init_tcr();
        init_page_tables();
        init_ttbr0();
        enable_mmu();
    }

    // C‑style null‑terminated string
    let msg = b"Hello from Rust kernel with MMU enabled!\n\0";
    uart_puts(msg.as_ptr());
}

// -----------------------------------------------------------------------------
// Panic handler (required in no_std)
// -----------------------------------------------------------------------------
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// -----------------------------------------------------------------------------
// Minimal C runtime shims required by libcore
// -----------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while i < n {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    }
    dst
}

#[no_mangle]
pub extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n {
            let da = *a.add(i);
            let db = *b.add(i);
            if da != db {
                return da as i32 - db as i32;
            }
            i += 1;
        }
    }
    0
}

// -----------------------------------------------------------------------------
// Exception vector initialization (sets VBAR_EL1 to point to our exception vectors)
// -----------------------------------------------------------------------------
unsafe fn init_exceptions() {
    extern "C" {
        static exception_vectors: u8;
    }

    let addr = unsafe { &exception_vectors as *const _ as u64 };

    unsafe {
        core::arch::asm!(
            "msr VBAR_EL1, {0}",
            in(reg) addr,
            options(nostack, preserves_flags),
        );
    }
}

// -----------------------------------------------------------------------------
// MAIR initialization (sets up memory attributes for normal and device memory)
// -----------------------------------------------------------------------------
unsafe fn init_mair() {
    let mair_value: u64 =
        (0x00 << 0)  |   // Attr0 = Device-nGnRnE
        (0x44 << 8)  |   // Attr1 = Normal Non-Cacheable
        (0xFF << 16);    // Attr2 = Normal Write-Back Cacheable

    core::arch::asm!(
        "msr MAIR_EL1, {0}",
        in(reg) mair_value,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// TCR initialization (sets up translation control for 48-bit VA space, 4KB granules, etc.)
// -----------------------------------------------------------------------------
unsafe fn init_tcr() {
    let tcr_value: u64 =
        (16 << 0) |        // T0SZ = 16 → 48-bit VA space
        (0b00 << 14) |     // TG0 = 4KB granule
        (0b11 << 12) |     // SH0 = Inner Shareable
        (0b01 << 10) |     // ORGN0 = Write-back cacheable
        (0b01 << 8)  |     // IRGN0 = Write-back cacheable
        (0b101 << 32);     // IPS = 48-bit physical address size

    core::arch::asm!(
        "msr TCR_EL1, {0}",
        in(reg) tcr_value,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// Minimal page tables initialization (maps 0x4008_0000 for kernel and 0x0900_0000 for UART)
// -----------------------------------------------------------------------------
// Helper function to create a block entry in the page table
fn l1_block_entry(phys: u64, attr_index: u64, executable: bool) -> u64 {
    let mut desc =
        (phys & !((1u64 << 30) - 1)) |   // Align physical address to 1 GB // use 0x0000_FFFF_FFFF_0000 or !((1<<12)-1) for 4KB pages alignment (L3), !((1<<21)-1) for 2MB blocks (L2)
        (attr_index << 2) |              // AttrIndx
        (1 << 10) |                      // AF
        (3 << 8)  |                      // SH = Inner Shareable
        (0 << 6)  |                      // AP = RW EL1
        //(1 << 54) |                       // PXN
        //(1 << 53) |                       // UXN
        0b01;                            // VALID + BLOCK

    if !executable {
        // PXN + UXN = Execute Never for both privileged and unprivileged
        desc |= (1 << 54) | (1 << 53);
    }

    desc
}

// Helper function to create a block entry in the L2 page table (maps 2 MB blocks)
fn l2_block_entry(phys: u64, attr_index: u64, executable: bool) -> u64 {
    let mut desc =
        (phys & !((1u64 << 21) - 1)) | // align 2 Mo
        (attr_index << 2) |
        (1 << 10) |
        (3 << 8)  |
        (0 << 6)  |
        0b01; // VALID + BLOCK (L2)

    if !executable {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}
/*
// Maps the given physical address range [start, end) in the given L2 page table with 2 MB blocks
unsafe fn map_section_l2(
    l2: &mut PageTable,
    base_phys: u64,
    start: u64,
    end: u64,
    executable: bool,
) {
    let first = ((start - base_phys) >> 21) as usize;
    let last  = ((end   - base_phys) >> 21) as usize;

    for i in first..=last {
        let phys = base_phys + ((i as u64) << 21);
        l2.0[i] = l2_block_entry(phys, 2, executable);
    }
}

// Helper function to create a page entry in the L3 page table (maps 4 KB pages)
fn l3_page_entry(phys: u64, attr_index: u64, executable: bool) -> u64 {
    let mut desc =
        (phys & !((1u64 << 12) - 1)) | // align 4K
        (attr_index << 2) |
        (1 << 10) |  // AF
        (3 << 8)  |  // SH = Inner Shareable
        (0 << 6)  |  // AP = RW EL1
        0b11;        // VALID + PAGE (L3)

    if !executable {
        desc |= (1 << 54) | (1 << 53); // PXN + UXN
    }

    desc
}

// Maps the given physical address range [start, end) in the given L3 page table with 4 KB pages
unsafe fn map_section_l3(
    l3: *mut PageTable,
    base_phys: u64,
    start: u64,
    end: u64,
    executable: bool,
) {
    let first = ((start - base_phys) >> 12) as usize;
    let last  = ((end   - base_phys) >> 12) as usize;

    for i in first..=last {
        let phys = base_phys + ((i as u64) << 12);
        (*l3).0[i] = l3_page_entry(phys, 2, executable);
    }
}
*/
unsafe fn init_page_tables() {
    // L0[0] -> L1
    L0_TABLE.0[0] = (&raw const L1_TABLE as *const _ as u64) | 0b11;

    // L1[0]: 0–1 Go: identity mapping of the first 1 Go of physical memory (where the kernel and peripherals are located) in Normal WB, RW, executable
    let phys0 = 0u64 << 30;
    L1_TABLE.0[0] = l1_block_entry(phys0, 2, true);

    // L1[1]: 1–2 Go -> table L2_KERNEL
    L1_TABLE.0[1] = (&raw const L2_KERNEL_TABLE as *const _ as u64) | 0b11;

    // We fill L2_KERNEL_TABLE with 2 Mo blocks mapping the kernel region (0x4008_0000–0x7FFF_FFFF) in Normal WB, RW, executable
    // 1-2 Go region base physical address (where the kernel is loaded by the bootloader, defined in the linker script) – we assume the kernel is loaded at 1 Go (0x4000_0000) for simplicity, but in a real kernel you'd want to use the actual load address defined in your linker script
    let l1_1_base_phys = 1u64 << 30; // 0x4000_0000
    // Start/End kernel virtual addresses (identical to physical since on identity mapping + Virtual Address = Physical Address for 1–2 Go) defined in the linker script
    let kernel_start = &_kernel_start as *const u8 as u64;
    let kernel_end   = &_kernel_end   as *const u8 as u64;
    // We compute the first and last L2 indices that cover the kernel region
    // L1[1] covers 0x4000_0000–0x7FFF_FFFF
    // L2 index = bits [29:21]
    let first_l2 = (((kernel_start - l1_1_base_phys) >> 21) & 0x1FF) as usize;
    let last_l2  = (((kernel_end   - l1_1_base_phys) >> 21) & 0x1FF) as usize;
    for i in first_l2..=last_l2 {
        let phys = l1_1_base_phys + ((i as u64) << 21); // 2 Mo
        // For now: we mark the whole 1–2 Go region as executable, but in a real kernel you'd want to mark only the actual code sections as executable and the rest (data, bss, peripherals) as non-executable
        L2_KERNEL_TABLE.0[i] = l2_block_entry(phys, 2, true);
    }

    /*
    // L2[0] -> L3_KERNEL)
    L2_KERNEL_TABLE.0[0] = (&raw const L3_KERNEL_TABLE as *const _ as u64) | 0b11;
    // we now have:
    // VA = 0x4008_0000
    // base L1[1] = 0x4000_0000
    // offset = 0x80000
    // offset >> 21 = 0 -> L2 index 0

    // Map L3
    
    let base = 1u64 << 30; // 0x4000_0000

    // .text → executable
    map_section_l3(
        &raw mut L3_KERNEL_TABLE,
        base,
        &_text_start as *const u8 as u64,
        &_text_end   as *const u8 as u64,
        true,
    );

    // .rodata → non executable
    map_section_l3(
        &raw mut L3_KERNEL_TABLE,
        base,
        &_rodata_start as *const u8 as u64,
        &_rodata_end   as *const u8 as u64,
        false,
    );

    // .data → non executable
    map_section_l3(
        &raw mut L3_KERNEL_TABLE,
        base,
        &_data_start as *const u8 as u64,
        &_data_end   as *const u8 as u64,
        false,
    );

    // .bss → non executable
    map_section_l3(
        &raw mut L3_KERNEL_TABLE,
        base,
        &_bss_start as *const u8 as u64,
        &_bss_end   as *const u8 as u64,
        false,
    );
    */
}

// -----------------------------------------------------------------------------
// TTBR0 initialization (sets TTBR0_EL1 to point to our L0 page table)
// -----------------------------------------------------------------------------
unsafe fn init_ttbr0() {
    let l0_addr = &raw const L0_TABLE as *const _ as u64;

    core::arch::asm!(
        "msr TTBR0_EL1, {0}",
        in(reg) l0_addr,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// Enable MMU (sets SCTLR_EL1 to enable MMU, caches, etc.)
// -----------------------------------------------------------------------------
unsafe fn enable_mmu() {
    let mut sctlr: u64;

    core::arch::asm!(
        "mrs {0}, SCTLR_EL1",
        out(reg) sctlr,
        options(nostack, preserves_flags),
    );

    // Enable MMU (M), data cache (C), and instruction cache (I)
    sctlr |= 1 << 0;   // M
    sctlr |= 1 << 2;   // C
    sctlr |= 1 << 12;  // I

    core::arch::asm!(
        "msr SCTLR_EL1, {0}",
        "isb",
        in(reg) sctlr,
        options(nostack, preserves_flags),
    );
}

// -----------------------------------------------------------------------------
// UART driver (MMIO at 0x0900_0000)
// -----------------------------------------------------------------------------
#[inline(always)]
fn uart_putc(c: u8) {
    unsafe {
        let uart = 0x0900_0000 as *mut u32;
        *uart = c as u32;
    }
}

#[inline(always)]
fn uart_puts(s: *const u8) {
    unsafe {
        let mut p = s;
        loop {
            let c = *p;
            if c == 0 {
                break;
            }
            uart_putc(c);
            p = p.add(1);
        }
    }
}
