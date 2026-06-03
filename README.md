![Status](https://img.shields.io/badge/status-experimental-orange)

# WOS — W. Operating System

**WOS** is an experimental operating system written in **Rust** and built **from scratch** for the **ARM64/AArch64** architecture.

At its current stage, WOS is a **minimal, clean, and educational kernel**, implementing the essential foundations of a modern OS:

- custom ARM64 boot pipeline  
- EL1 CPU initialization  
- full MMU setup (MAIR, TCR, TTBR0, SCTLR)  
- L0/L1/L2 page tables  
- UART output  
- physical page allocator  
- Rust `no_std` / `no_main` environment  

This project stands out because it is **Rust-first** and **ARM64-first**, a combination that is still extremely rare in the OS development ecosystem.

---

## 🎯 Project Vision

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

### 2. **A future platform for an AI‑native operating system**
Long-term, WOS aims to explore the idea of an AI‑native operating system.
The AI components are not part of the public codebase yet. 

---

## ✨ Current Features

- Minimal Rust kernel (`no_std`, `no_main`)
- Custom AArch64 bootloader and startup code
- UART driver (console output)
- MMU enabled with full configuration
- L0/L1/L2 page tables
- Physical page allocator (4K pages)
- Hexadecimal debug output
- QEMU‑friendly environment

---

## 📁 Project Structure

- `/kernel` — Rust kernel + assembly entrypoint + linker script  
- `/boot` — Early AArch64 boot code  
- `/docs` — Technical documentation  
- `/build` — Build artifacts (ignored by Git)

---

## 🛠️ Build Instructions

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

```bash
cd kernel
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

#### GIC Version
This kernel uses **GICv2**, *not GICv3*. Indeed, the default QEMU `virt` machine exposes a GIC compatible with:
```bash
compatible = "arm,cortex-a15-gic";
```

This corresponds to **GICv2/GIC-400**, the interrupt controller used in Cortex‑A15‑class systems.

No GICv3 redistributors (`GICR`) or ITS nodes are present in the Device Tree.

#### Memory Mapping (from the DTB)
The DTB provides the following `reg` entries for the interrupt controller:
```bash
reg = <0x00000000 0x08000000   // GICD base
       0x00000000 0x00010000   // GICD size
       0x00000000 0x08010000   // GICC base
       0x00000000 0x00010000>; // GICC size
```

Therefore:
- GICD_BASE = 0x08000000
- GICC_BASE = 0x08010000

These addresses are used by the kernel to initialize the distributor and CPU interface.

#### Important Note About QEMU
If you run QEMU with:
```bash
-machine virt
```
You get **GICv2** (default).

If you run QEMU with:
```bash
-machine virt,gic-version=3
```
You get **GICv3**, which this kernel does not support.

Attempting to boot with GICv3 will result in crashes (invalid register accesses, data aborts, etc.).

---

## 📌 Current Status

WOS is not yet a full operating system.
It is a functional minimal kernel, serving as a foundation for:
- advanced memory management
- interrupts and timers
- a basic scheduler
- a user space
- an AI‑native runtime

## 🗺️ Roadmap (Kernel)

- [x] ARM64 boot + Rust kernel
- [x] UART output
- [x] MMU + page tables
- [x] Physical page allocator
- [ ] Interrupts + timer
- [ ] Virtual memory allocator (heap)
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