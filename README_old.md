![Status](https://img.shields.io/badge/status-experimental-orange)

# WOS — Wasserstein Operating System

WOS is an experimental operating system designed around a native AI engine based on optimal transport theory, including Wasserstein distances and Sinkhorn algorithms.

The goal is to explore new architectures where machine learning primitives are first‑class citizens of the kernel.

For the theoretical foundations behind the AI-native memory system used in WOS, see the companion research paper:  
https://arxiv.org/abs/2604.16052

---

## Features (Work in Progress)

- Minimal Rust-based ARM64 kernel
- Custom ARM64 bootloader and early startup code
- AI-Native core using optimal transport
- Modular architecture for experimentation
- QEMU-friendly development environment

---

## Project Structure

- `/kernel` — Rust Kernel (no_std, no_main)  
- `/boot` — Early boot code (AArch64)  
- `/build` — Build artifacts (ignored by Git)  
- `/docs` — Technical documentation  

The kernel lives entirely under `/kernel/`, including the assembly entrypoint, linker script, and Rust sources.

---

## Build Instructions

Make sure Rust (nightly) and the required AArch64 toolchains are installed.

From the project root:

```bash
cd kernel
cargo build
```

This produces the kernel ELF at:

```bash
kernel/target/aarch64-wos/debug/kernel
```

---

## Run in QEMU (ARM64)

```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -kernel kernel/target/aarch64-wos/debug/kernel \
    -nographic
```

---

## Status

This project is in early development and not yet functional as a full OS.

The current focus area include:
- building a minimal Rust kernel from scratch
- validating the boot pipeline
- implementing UART output
- preparing the foundation for AI‑native memory structures

---

## Roadmap

- [ ] Minimal bootloader (ARM64)
- [ ] Rust kernel skeleton
- [ ] Memory manager
- [ ] Interrupt handling
- [ ] Wasserstein core module
- [ ] Sinkhorn solver integration
- [ ] QEMU test harness

---

## License

Private project — all rights reserved.