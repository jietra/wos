# WOS — W. Operating System
![Status](https://img.shields.io/badge/status-kernel_ready-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)

**WOS** is an experimental operating system written in **Rust**, built **from scratch**, now supporting **ARM64/AArch64** *and* **RISC-V (rv64)** architectures.

The kernel is stable enough for experimentation and educational use, but **not intended for production environments**.

At its current stage, WOS is a **minimal, clean, and educational kernel**, implementing the essential foundations of a modern OS:

- custom ARM64 and RISC-V boot pipeline  
- EL1 CPU initialization (ARM64)
- early bring‑up on RISC‑V (stack + Rust entry)
- full ARM64 MMU setup (MAIR, TCR, TTBR0, SCTLR)
- hierarchical L0/L1/L2/L3 page tables (ARM64)
- per‑section kernel mapping
- UART output  
- physical page allocator  
- Rust `no_std` / `no_main` environment  

WOS is **Rust-first**, **multi-architecture**, and designed to be **educational and clean**.

## Table of Contents
- [Project Vision](#-project-vision)
- [Multi-Architecture Support](#-multi-architecture-support)
- [Current Features](#-current-features)
- [Project Structure](#-project-structure)
- [Prerequisites](#-prerequisites)
- [Build Instructions](#build-instructions-)
- [Run in QEMU (ARM64)](#️-run-in-qemu-arm64)
- [Run in QEMU (RISC-V)](#️-run-in-qemu-risc-v)
- [Current Status](#-current-status)
- [Roadmap](#️-roadmap-kernel)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🎯 Project Vision

WOS is a research‑driven exploration of how operating‑system foundations might evolve in an era where adaptive, learning‑based components become pervasive.

WOS has two complementary goals:

### 1. **A minimal and educational Rust OS (ARM64 + RISC-V)**
WOS provides a clear, modern, and understandable foundation for learning:

- how to boot ARM64 and RISC-V CPUs    
- how to write a Rust kernel without a runtime  
- how to manage physical memory  
- how to interact with hardware (UART, timers, etc.)
- how to configure the ARM64 MMU
- how to bring up a minimal RISC‑V kernel

This makes WOS useful for:

- systems programming students  
- Rust developers curious about OS internals  
- people wanting a simple ARM64 or RISC-V kernel  
- researchers needing a minimal multi-arch environment  

> Status (ARM64): WOS now includes a fully correct ARMv8 MMU with fine‑grained 4 KiB mappings, making it a solid foundation for advanced kernel features such as user space, virtual memory, and process isolation.


### 2. **A future platform for an AI‑native operating system**
WOS is also the foundation for a broader research direction exploring the idea of an AI‑native OS.

> Note: The AI components are not part of the public codebase. 

---

## 🏗️ Multi‑Architecture Support
WOS now supports two architectures:

### ✔ ARM64 / AArch64
Fully functional:
- custom bootloader
- EL1 initialization
- MMU + page tables
- UART
- GICv2
- physical memory allocator

### ✔ RISC‑V (rv64)
Early bring‑up:
- custom bootloader (start.S)
- stack setup
- Rust entry (rust_main)
- UART “Hello” output
- working linker script
- custom target JSON
- correct code model (medium)
- Architecture‑specific code lives in:

```
kernel/src/arch/
  ├── aarch64/
  └── riscv64/
```
---

## ✨ Current Features

- Minimal Rust kernel (`no_std`, `no_main`)
- Custom boot code for ARM64 and RISC‑V
- UART driver (console output)
- ARM64 exception handling (synchronous exceptions, data aborts, FP/SIMD traps)
- ARMv8 MMU setup (MAIR, TCR, TTBR0, SCTLR)
- 4‑level translation tables (L0/L1/L2/L3)
- Fine‑grained 4 KiB kernel mapping (text/rodata/data/bss/stack)
- Physical page allocator (4K pages)
- Debug helpers
- QEMU‑friendly environment
- Basic GICv2 initialization (Distributor + CPU interface), currently using identity-mapped MMIO (ARM64)

> **Note (ARM64):**  
> Device MMIO is currently accessed through identity-mapped physical addresses.
> This simplifies early bring-up of interrupts and timers.  
> A full high-half kernel layout (KERNEL_BASE / DEVICE_BASE in the 0xFFFF_… range) will be enabled once IRQs, timers, and the scheduler are stable.

---

## 📁 Project Structure

```
/docs                   # Technical documentation
/kernel
  ├── src
  │   ├── arch/aarch64  # ARM64-specific code
  │   ├── arch/riscv64  # RISC-V-specific code
  │   ├── memory        # Memory management
  │   ├── drivers       # UART, future DTB parsing, etc.
  │   ├── debug         # Debug helpers
  │   ├── utils         # Printing, helpers
  │   └── main.rs       # Kernel entry point (Rust)
  ├── linker/           # Linker scripts per architecture
  ├── targets/          # Custom Rust target JSON files
  ├── build.rs          
  └── virt.dtb          # QEMU DTB (ARM64)
```

---

## 🧰 Prerequisites
WOS requires:
- Rust nightly
- clang / LLVM (for assembling start.S via build.rs)
- QEMU with ARM64 and RISC‑V support

On Debian/Ubuntu:
```bash
sudo apt install clang llvm qemu-system-arm qemu-system-misc
```

On macOS (M chips), using `UTM` is recommended for convenience and stability. WOS runs perfectly inside a `UTM` ARM64 virtual machine.

---

## 🛠️ Build Instructions

### Build for ARM64
```bash
cd kernel
cargo build --target targets/aarch64-wos.json
```

### Build for RISC‑V
```bash
cd kernel
cargo build --target targets/riscv64-wos.json
```

---

## ▶️ Run in QEMU (ARM64)
Generate the QEMU virt machine DTB once in the `kernel` Directory:
```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -machine dumpdtb=virt.dtb \
    -nographic
```

Run the kernel:
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

> Note (ARM64): Interrupt Controller (GIC)
>
> WOS now includes **working GICv2 initialization** (Distributor + CPU interface). At this stage, the kernel still uses **identity-mapped physical addresses** for MMIO (e.g., UART at 0x0900_0000, GICD at 0x0800_0000), because the high-half virtual address space (DEVICE_BASE at 0xFFFF_FD00_0000_0000) is not enabled yet.
>
> This is intentional: interrupts, timers, and exception handling are brought up first in a simple identity-mapped environment before enabling the full high-half kernel with TTBR1 and virtualized device mappings.

## ▶️ Run in QEMU (RISC‑V)
Run **without OpenSBI** (`-bios none`):

```bash
qemu-system-riscv64 \
    -M virt \
    -cpu rv64 \
    -kernel target/riscv64-wos/debug/kernel \
    -nographic \
    -bios none
```

---

## 📌 Current Status

WOS is not yet a full operating system.
It is a functional minimal kernel, serving as a foundation for:
- advanced memory management
- interrupts and timers
- a basic scheduler
- a user space
- an AI‑native runtime

ARM64 is fully functional; RISC‑V is in early bring‑up.

## 🗺️ Roadmap

### ARM64

- [x] boot + Rust kernel
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

### RISC‑V

- [x] Boot + Rust entry
- [x] UART “Hello”
- [x] Working linker script
- [x] Custom target JSON
- [ ] Trap handler
- [ ] Sv39 MMU
- [ ] Hart management
- [ ] Device tree parsing
- [ ] UART + timer drivers
- [ ] Virtual memory
- [ ] Scheduler

---

## 🤝 Contributing
Contributions are welcome — especially in the areas of:
- ARM64 architecture & low‑level bring‑up
- RISC‑V bring‑up
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
All publicly available components of WOS are released under the **MIT License**.  
This makes the kernel freely usable for learning, experimentation, and derivative work.

### **2. AI‑native components — Proprietary**
The AI‑native runtime and related modules are **not included in this repository** and remain **closed‑source and proprietary**.  
They will be distributed separately and are not covered by the MIT license.

See the [LICENSE](LICENSE) file for details.