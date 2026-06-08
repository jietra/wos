# WOS — W. Operating System
![Status](https://img.shields.io/badge/status-kernel_ready-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

**WOS** is an experimental operating system written in **Rust** and built **from scratch** for the **ARM64/AArch64** architecture.

The kernel is stable enough for experimentation and educational use, but **not intended for production environments**.

At its current stage, WOS is a **minimal, clean, and educational kernel**, implementing the essential foundations of a modern OS:

- custom ARM64 boot pipeline  
- EL1 CPU initialization  
- full MMU setup (MAIR, TCR, TTBR0, SCTLR)
- hierarchical L0/L1/L2/L3 page tables
- per‑section kernel mapping:
  - .text → RX, RO
  - .rodata → R, XN
  - .data/.bss → RW, XN
  - stack → RW, XN  
- UART output  
- physical page allocator  
- Rust `no_std` / `no_main` environment  

This project stands out because it is **Rust-first** and **ARM64-first**, a combination that is still extremely rare in the OS development ecosystem.


## Table of Contents
- [Project Vision](#-project-vision)
- [Current Features](#-current-features)
- [Project Structure](#-project-structure)
- [Build Instructions](#build-instructions-)
- [Run in QEMU](#️-run-in-qemu-arm64)
- [Current Status](#-current-status)
- [Roadmap](#️-roadmap-kernel)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🎯 Project Vision

WOS is a research‑driven exploration of how operating‑system foundations might evolve in an era where adaptive, learning‑based components become pervasive.

WOS has two complementary goals:

### 1. **A minimal and educational Rust ARM64 OS**
WOS provides a clear, modern, and understandable foundation for learning:

- how to boot an ARM64 CPU  
- how to enable and configure the MMU  
- how to write a Rust kernel without a runtime  
- how to manage physical memory  
- how to interact with hardware (UART, timers, etc.)

This makes WOS useful for:

- systems programming students  
- Rust developers curious about OS internals  
- people wanting a simple ARM64 kernel for Raspberry Pi or QEMU  
- researchers needing a minimal ARM64 environment  
- developers exploring unikernels in Rust  

> Status: WOS now includes a fully correct ARMv8 MMU with fine‑grained 4 KiB mappings, making it a solid foundation for advanced kernel features such as user space, virtual memory, and process isolation.


### 2. **A future platform for an AI‑native operating system**
WOS is not only a minimal ARM64 kernel. It is also the foundation for a broader research direction. Long-term, WOS aims to explore the idea of an AI‑native operating system.

> Note: The AI components are not part of the public codebase. 

---

## ✨ Current Features

- Minimal Rust kernel (`no_std`, `no_main`)
- Custom AArch64 bootloader and startup code
- UART driver (console output)
- Full exception handling (synchronous exceptions, data aborts, FP/SIMD traps)
- Full ARMv8 MMU setup (MAIR, TCR, TTBR0, SCTLR)
- 4‑level translation tables (L0/L1/L2/L3)
- Fine‑grained 4 KiB kernel mapping (text/rodata/data/bss/stack)
- Correct memory attributes (Normal WB, Device-nGnRnE, XN, RO/RW)
- Physical page allocator (4K pages)
- Hexadecimal debug output
- QEMU‑friendly environment
- Basic GICv2 initialization (Distributor + CPU interface), currently using identity-mapped MMIO

> **Note:**  
> Device MMIO is currently accessed through identity-mapped physical addresses.
> This simplifies early bring-up of interrupts and timers.  
> A full high-half kernel layout (KERNEL_BASE / DEVICE_BASE in the 0xFFFF_… range) will be enabled once IRQs, timers, and the scheduler are stable.

---

## 📁 Project Structure

```bash
/docs                   # Technical documentation
/kernel
  ├── src
  │   ├── arch/aarch64  # Architecture-specific code
  │   ├── mmu           # Page tables and MMU setup
  │   ├── memory        # Physical memory management and allocators
  │   ├── drivers       # UART, future DTB parsing, etc.
  │   ├── debug         # Debug helpers (CPU, memory tests, DTB debug)
  │   ├── utils         # Printing, helpers
  │   └── main.rs       # Kernel entry point (Rust)
  ├── linker.ld         # Linker script
  └── virt.dtb          # QEMU virt machine DTB (generated)
```

---

## Build Instructions 🛠️

The kernel now requires a 4‑level MMU‑capable ARMv8 CPU (all QEMU virt machines support this).

Ensure Rust nightly and the required ARM64 toolchains are installed.

```bash
cd kernel
cargo build
````
The kernel ELF is produced at:

```bash
kernel/target/aarch64-wos/debug/kernel
```

---

## ▶️ Run in QEMU (ARM64)
On macOS, using UTM is recommended for convenience and stability.
WOS runs perfectly inside a UTM ARM64 virtual machine.

Generate the QEMU virt machine DTB in the `kernel` Directory (generate only once):
```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -machine dumpdtb=virt.dtb \
    -nographic
```

Run the kernel with the generated DTB, e.g. in `kernel` Directory:
```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -kernel target/aarch64-wos/debug/kernel \
    -dtb virt.dtb \
    -nographic
```

You may also build an image,
```bash
rust-objcopy --strip-all -O binary target/aarch64-wos/debug/kernel kernel8.img
```
and run the kernel from this image:
```bash
qemu-system-aarch64   -M virt   -cpu cortex-a72   -kernel kernel8.img   -nographic
```

### 📝 Interrupt Controller (GIC)

WOS now includes **working GICv2 initialization** (Distributor + CPU interface). At this stage, the kernel still uses **identity-mapped physical addresses** for MMIO (e.g., UART at 0x0900_0000, GICD at 0x0800_0000), because the high-half virtual address space (DEVICE_BASE at 0xFFFF_FD00_0000_0000) is not enabled yet.

This is intentional: interrupts, timers, and exception handling are brought up first in a simple identity-mapped environment before enabling the full high-half kernel with TTBR1 and virtualized device mappings.

---

## 📌 Current Status

WOS is not yet a full operating system.
It is a functional minimal kernel, serving as a foundation for:
- advanced memory management
- interrupts and timers
- a basic scheduler
- a user space
- an AI‑native runtime

The interrupt controller (GICv2) is now initialized and functional, using identity-mapped MMIO as a temporary bring-up strategy.

## 🗺️ Roadmap (Kernel)

- [x] ARM64 boot + Rust kernel
- [x] UART output
- [x] MMU + page tables
- [x] Physical page allocator
- [x] Basic GICv2 bring-up (Distributor + CPU interface)
- [ ] Timer interrupts (CNTV)
- [ ] High-half virtual device mapping (UART, GIC, timers) using TTBR1 + DEVICE_BASE
- [ ] Virtual memory allocator (heap, using L3 pages)
- [ ] Minimal scheduler
- [ ] Drivers (UART, timer, virtio)
- [ ] User space
- [ ] ELF loader

---

## 🤝 Contributing
WOS is still in an early experimental stage, but contributions are welcome — especially in the areas of:
- ARM64 architecture & low‑level bring‑up
- Rust no_std development
- memory management
- exception handling & timers
- device drivers (UART, timer, virtio)
- documentation & educational material

Before contributing, please keep in mind:

### ✔️ Kernel contributions must follow the MIT license
All code submitted to this repository will be licensed under the MIT License, consistent with the rest of the kernel.

### ✔️ AI‑native components are not open to contribution
The AI‑native runtime and related modules are **not part of this repository** and remain closed‑source and proprietary.
They are developed separately and are **not open to direct code contributions**.

However, **high‑level discussions, conceptual feedback, and private exchanges about the AI‑native architecture are welcome**.
If you are interested in the research direction or want to discuss ideas, feel free to open an issue or reach out privately.

### ✔️ Code style & expectations
- Rust nightly
- `no_std`, `no_main`
- minimal dependencies
- clear, well‑commented low‑level code
- small, focused pull requests

### ✔️ How to contribute
1. Fork the repository
2. Create a feature branch
3. Submit a pull request with a clear description
4. Keep the scope minimal and focused

If you're unsure whether a contribution fits the project, feel free to open an issue first.

---

## 📜 License
WOS uses a **dual licensing model**:

### **1. Kernel (this repository) — MIT License**
All publicly available components of WOS — including the Rust ARM64 kernel, boot code, and supporting infrastructure — are released under the **MIT License**.  
This makes the kernel freely usable for learning, experimentation, and derivative work.

### **2. AI‑native components — Proprietary**
The AI‑native runtime and related modules are **not included in this repository** and remain **closed‑source and proprietary**.  
They will be distributed separately and are not covered by the MIT license.

See the [LICENSE](LICENSE) file for details.