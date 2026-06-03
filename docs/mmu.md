# ARMv8 MMU and Paging in WOS

WOS implements a **fully functional ARMv8 MMU setup** for AArch64 EL1, with:
- MAIR_EL1 configuration (Normal vs Device memory)
- TCR_EL1 configuration (4‑level translation, 4 KiB pages)
- TTBR0_EL1 pointing to the kernel page tables
- SCTLR_EL1 enabling the MMU and caches
- 4‑level translation tables: L0 → L1 → L2 → L3
- fine‑grained 4 KiB mappings for the kernel

This document describes how virtual memory is organized and how the kernel is mapped.

---

## 1. Virtual Address Space Layout

WOS currently uses a **simple identity mapping** for the kernel:
- virtual address = physical address (for the kernel region)
- the kernel is loaded by the bootloader at **0x4008_0000**
- the 1–2 GiB region (0x4000_0000–0x7FFF_FFFF) is used for the kernel

This keeps the design simple while still exercising the full MMU machinery.

---

## 2. Translation Levels

WOS uses the standard 4‑level ARMv8 translation scheme with 4 KiB pages:
- **L0**: top‑level table (512 entries)
- **L1**: covers 1 GiB regions
- **L2**: covers 2 MiB regions
- **L3**: covers 4 KiB pages

The relevant tables are defined in `kernel/src/mmu/tables.rs`:

```rust
// kernel/src/mmu/tables.rs

#[repr(align(4096))]
pub struct PageTable(pub [u64; 512]);

pub static mut L0_TABLE: PageTable = PageTable([0; 512]);
pub static mut L1_TABLE: PageTable = PageTable([0; 512]);
pub static mut L2_TABLE: PageTable = PageTable([0; 512]);
pub static mut L3_KERNEL_TABLE: PageTable = PageTable([0; 512]);
```

---

## 3. High-Level Mapping

The current mapping is:
- **L0[0]** → L1
- **L1[0]** → 0–1 GiB (Device, for peripherals)
- **L1[1]** → 1–2 GiB (kernel region) → L2
- **L2[kernel]** → 2 MiB block containing the kernel → L3_KERNEL_TABLE
- **L3_KERNEL_TABLE[0..511]** → 4 KiB pages with per‑section attributes

This gives a **single 2 MiB region** for the kernel, split into 512 pages of 4 KiB.

---

## 4. Memory Attributes (MAIR_EL1)

WOS configures three memory attribute slots in `MAIR_EL1`:
- **Attr0 = 0x00** — Device-nGnRnE  
  Used for MMIO regions (UART, GIC, timers).
- **Attr1 = 0x44** — Normal, Non‑Cacheable  
  Reserved for future use (not currently referenced in page tables).
- **Attr2 = 0xFF** — Normal, Write‑Back, Read/Write Allocate  
  Used for all kernel sections (`.text`, `.rodata`, `.data`, `.bss`, stack).

The page table entries reference these attributes via the `AttrIndx` field.

---

## 5. Kernel Section Mapping

The kernel linker script exposes the following symbols (see `kernel/linker` and `kernel/src/arch/aarch64/linker_symbols.rs`):

```Rust
extern "C" {
    pub static _text_start: u8;
    pub static _text_end: u8;
    pub static _rodata_start: u8;
    pub static _rodata_end: u8;
    pub static _data_start: u8;
    pub static _data_end: u8;
    pub static _bss_start: u8;
    pub static _bss_end: u8;
}
```

WOS uses these to assign **different attributes per section** inside a single 2 MiB L2 region.

The mapping policy is:
- **.text**
    - Attr: Normal WB (index 2)
    - Access: EL1 Read‑Only (AP = RO)
    - Exec: executable (XN = 0)
- **.rodata**
    - Attr: Normal WB
    - Access: EL1 Read‑Only
    - Exec: non‑executable (XN = 1)
- **.data / .bss**
    - Attr: Normal WB
    - Access: EL1 Read‑Write
    - Exec: non‑executable
- **stack**: Located in `.bss`, therefore inherits `.bss` attributes:
    - Attr: Normal WB
    - Access: EL1 Read‑Write
    - Exec: non‑executable

> Note: WOS currently assumes that all kernel sections fit within a single 2 MiB L2 region. This is true for the current minimal kernel, but future growth may require multiple L2 entries.

This is implemented in `map_kernel_l3()`:

```Rust
// kernel/src/mmu/tables.rs (simplified)

use crate::arch::aarch64::linker_symbols as ls;

pub unsafe fn map_kernel_l3() {
    let l1_1_base = 1u64 << 30; // 0x4000_0000

    let text_start   = &ls::_text_start   as *const u8 as u64;
    let text_end     = &ls::_text_end     as *const u8 as u64;
    let rodata_start = &ls::_rodata_start as *const u8 as u64;
    let rodata_end   = &ls::_rodata_end   as *const u8 as u64;
    let data_start   = &ls::_data_start   as *const u8 as u64;
    let data_end     = &ls::_data_end     as *const u8 as u64;
    let bss_start    = &ls::_bss_start    as *const u8 as u64;
    let bss_end      = &ls::_bss_end      as *const u8 as u64;

    // Assume all sections fit in the same 2 MiB region
    let l2_index = (((text_start - l1_1_base) >> 21) & 0x1FF) as usize;
    let l2_region_base = l1_1_base + ((l2_index as u64) << 21);

    // L2 -> L3
    L2_TABLE.0[l2_index] = (&raw const L3_KERNEL_TABLE as *const _ as u64) | 0b11;

    // Fill all 512 4 KiB pages
    for i in 0..512 {
        let va = l2_region_base + ((i as u64) << 12);
        let phys = va; // identity mapping

        let (attr_index, ap, exec) = if va >= text_start && va < text_end {
            (2, 0b10, true)   // .text : Normal WB, RO, executable
        } else if va >= rodata_start && va < rodata_end {
            (2, 0b10, false)  // .rodata : Normal WB, RO, XN
        } else if (va >= data_start && va < data_end) || (va >= bss_start && va < bss_end) {
            (2, 0b00, false)  // .data/.bss : Normal WB, RW, XN
        } else {
            (2, 0b00, false)  // default: RW, XN
        };

        L3_KERNEL_TABLE.0[i] = l3_page_entry(phys, attr_index, exec, ap);
    }
}
```

---

## 6. Device Memory

Currently, WOS maps the entire 0–1 GiB region as Device-nGnRnE using a single L1 block entry. This covers all MMIO regions (UART, GIC, timers). Future versions will map each MMIO region individually at L3 granularity.

---

## 7. Future Work

Planned MMU‑related improvements:
- separate kernel and user address spaces
- non‑identity mapping for the kernel
- per‑process page tables
- copy‑on‑write and demand paging
- guard pages around stacks
- dedicated virtual regions for MMIO, heap, and user space