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

## ▶️ Run in QEMU (ARM64)
On macOS, using UTM is recommended for convenience and stability.
WOS runs perfectly inside a UTM ARM64 virtual machine.

```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -kernel kernel/target/aarch64-wos/debug/kernel \
    -nographic
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