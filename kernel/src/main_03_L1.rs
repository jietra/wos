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
//static mut L2_TABLE: PageTable = PageTable([0; 512]);
//static mut L3_TABLE: PageTable = PageTable([0; 512]);

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

unsafe fn init_page_tables() {
    // 1. L0 → L1
    L0_TABLE.0[0] = (&raw const L1_TABLE as *const _ as u64) | 0b11;

    // L1[0]: 0–1 Go: peripheral + other devices (0x0000_0000–0x3FFF_FFFF) in Device memory, RW, PXN/UXN
    let phys0 = 0u64 << 30;
    L1_TABLE.0[0] = l1_block_entry(phys0, 2, true);

    // L1[1]: 1-2 Go: kernel (0x4008_0000–0x7FFF_FFFF) in Normal memory, RW, PXN/UXN
    let phys1 = 1u64 << 30;
    L1_TABLE.0[1] = l1_block_entry(phys1, 2, true);

    //// Map the first 2 GB of physical memory using 1 GB blocks in L1 (each L1 entry maps 1 GB)
    //for i in 0..2 {
    //    let phys = (i as u64) << 30; // Each L1 entry maps 1 GB, so shift by 30 bits to get the physical address
    //    L1_TABLE.0[i] = l1_block_entry(phys, 2, true); // AttrIndx = 2 (Normal WB)
    //}

    //// 2. L1 → L2
    //L1_TABLE.0[0] = (&raw const L2_TABLE as *const _ as u64) | 0b11;

    //// 3. L2 → L3
    //L2_TABLE.0[0] = (&raw const L3_TABLE as *const _ as u64) | 0b11;

    //// 4. L3 → Kernel (0x4008_0000) in Normal memory, RW, PXN/UXN
    //let kernel_phys = 0x4008_0000u64;
    //L3_TABLE.0[0] = 
    //    (kernel_phys & 0x0000_FFFF_FFFF_F000) | // Align physical address to 4KB
    //    (2 << 2) |   // AttrIndx = 2 (Normal WB)
    //    (1 << 10) |  // AF = Access Flag
    //    (3 << 8)  |  // SH = Inner Shareable
    //    (0 << 6)  |  // AP = RW EL1
    //    (1 << 54) |  // PXN = Privileged Execute Never
    //    (1 << 53) |  // UXN = Unprivileged Execute Never
    //    0b11;        // VALID + PAGE

    //// 5. L3 → UART (0x0900_0000) in Device memory, RW, PXN/UXN
    //let uart_phys = 0x0900_0000u64;
    //L3_TABLE.0[1] =
    //    (uart_phys & 0x0000_FFFF_FFFF_F000) |
    //    (0 << 2) |   // AttrIndx = 0 (Device)
    //    (1 << 10) |  // AF
    //    (3 << 8)  |  // SH = Inner Shareable
    //    (0 << 6)  |  // AP = RW
    //    (1 << 54) |  // PXN
    //    (1 << 53) |  // UXN
    //    0b11;
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
