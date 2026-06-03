# WOS Architecture Overview

WOS is a minimal ARM64 kernel written in Rust, designed to be clean, modern, and educational.  
This document provides a high‑level overview of the internal architecture of the kernel.

---

## 1. High‑Level Design

WOS follows a classic microkernel‑inspired structure:
- **Architecture‑specific code** (AArch64 bring‑up, registers, exceptions)
- **Memory subsystem** (MMU, page tables, physical allocator)
- **Drivers** (UART, DTB parsing, GIC in progress)
- **Kernel core** (entry point, panic handling, debug utilities)

The kernel runs in **AArch64 EL1**, with a fully enabled MMU and fine‑grained 4 KiB mappings.

---

## 2. Boot Flow

The boot sequence is:
1. **QEMU loads the kernel at 0x4008_0000**
2. Execution begins in `start.S`
3. CPU is switched to EL1
4. Stack is initialized
5. BSS is zeroed
6. Rust entry point (`main.rs`) is called
7. MMU is configured and enabled
8. UART is initialized
9. Kernel prints its first messages

The early bring‑up code lives in:
```bash
kernel/src/arch/aarch64/start.S
```

---

## 3. Exception Handling

WOS implements a complete exception pipeline:
- Vector table in `exception_vectors.S`
- Rust handlers in `exceptions.rs`
- Decoding of ESR_EL1, ELR_EL1, FAR_EL1
- Support for:
  - synchronous exceptions
  - data/instruction aborts
  - alignment faults
  - FP/SIMD traps (avoided by enabling FP/SIMD early in `arch/aarch64/start.S`)
  - SError
  - IRQ (GICv2 integration in progress)

See `docs/exceptions.md` for details.

---

## 4. Memory Management

The memory subsystem is composed of three components:

### 4.1 MMU and Page Tables

WOS uses:
- **4‑level translation tables** (L0/L1/L2/L3)
- **4 KiB pages**
- **per‑section kernel mapping**:
  - `.text` → RX, RO
  - `.rodata` → R, XN
  - `.data/.bss` → RW, XN
  - stack → RW, XN

The MMU code lives in:
```bash
kernel/src/mmu/tables.rs
kernel/src/mmu/mod.rs
```

See `docs/mmu.md` for a full explanation.

### 4.2 Physical Memory Allocator

WOS includes a simple **4 KiB bump allocator** for physical memory.
It is used for:
- page table allocation
- early kernel memory needs
- future heap allocator

Located in:
```bash
kernel/src/memory/phys.rs
```

This module also provides:
- `alloc_page()` — allocate a 4 KiB physical page
- `alloc_page_table()` — allocate a page suitable for page tables

### 4.3. Virtual Memory Helpers

The `virt.rs` module contains helper functions for:
- aligning addresses
- computing L0/L1/L2/L3 indices
- future virtual memory management

Located in:
```bash
kernel/src/memory/virt.rs
```

This module will later support:
- virtual memory regions
- kernel heap
- user‑space address spaces
- page table manipulation

---

## 5. Drivers

### 5.1 UART (PL011)

The UART driver provides:
- low‑level character output (`putc`)
- raw string output (`puts`)

Located in:
```bash
kernel/src/drivers/uart.rs
```

High‑level printing (`print!`, `println!`, hexadecimal output) is implemented in:
```bash
kernel/src/utils/print.rs
```

These functions build on top of the UART driver.

### 5.2 Device Tree (DTB)

WOS embeds the QEMU‑provided DTB at build time using `include_bytes!`.
A lightweight DTB parser is available for debugging purposes: it walks the FDT structure, prints nodes, properties, and `reg` values.

Located in:
```bash
kernel/src/debug/dtb.rs
```

A full DTB driver (extracting UART base address, GIC base addresses, and memory layout) will be implemented later in:
```bash
kernel/src/drivers/dtb.rs
```

### 5.3 GICv2 (in progress)

The interrupt controller driver will support:
- enabling IRQs
- timer interrupts
- per‑CPU interface configuration

Located in:
```bash
kernel/src/arch/aarch64/gic.rs
```

---

## 6. Utilities

Utility modules include:
- `print.rs` — UART‑backed `print!` macros
- `c_shims.rs` — panic handler, abort routines

---

## 7. Kernel Entry Point

The Rust entry point is:
```bash
kernel/src/main.rs
```

It is responsible for:
- initializing UART
- enabling the MMU
- printing boot banners
- running early tests
- preparing for interrupts and scheduling

---

## 8. Future Architecture

Planned components include:
- virtual memory allocator (heap)
- scheduler
- user‑space processes
- system calls (SVC)
- virtual file system
- async I/O
- AI‑native runtime (external, proprietary)

---

## 9. Directory Structure Summary

```bash
/docs                 # Technical documentation
/kernel
├── src
│   ├── arch/aarch64  # Architecture-specific code
│   ├── mmu           # Page tables and MMU setup
│   ├── memory        # Physical memory management
│   ├── drivers       # UART, future DTB parsing...
│   ├── debug         # Debug helpers (CPU, memory tests, DTB debug)
│   ├── utils         # Printing, helpers
│   └── main.rs       # Kernel entry point
├── linker.ld         # Linker script
└── virt.dtb          # QEMU DTB (generated)
```

---

## 10. Philosophy

WOS is designed to be:
- **minimal** — no unnecessary abstractions  
- **clean** — readable, modern Rust  
- **educational** — ideal for learning ARM64 internals  
- **extensible** — ready for user space and advanced features  