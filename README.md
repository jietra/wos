![Status](https://img.shields.io/badge/status-experimental-orange)

# WOS — Wasserstein Operating System

WOS is an experimental operating system designed around a native AI engine based on optimal transport theory, including Wasserstein distances and Sinkhorn algorithms.

The goal is to explore new architectures where machine learning primitives are first‑class citizens of the kernel.

For the theoretical foundations behind the AI-native memory system used in WOS, see the companion research paper:  
https://arxiv.org/abs/2604.16052

## Features (Work in Progress)

- Minimal Rust-based kernel
- Custom ARM64 bootloader
- Native AI core using optimal transport
- Modular architecture for experimentation
- QEMU-friendly development environment

## Project Structure

- `/src` — Kernel source code  
- `/boot` — Bootloader and early startup code  
- `/build` — Build artifacts (ignored by Git)  
- `/docs` — Technical documentation  

## Build Instructions

Make sure Rust and the required toolchains are installed.


## Run in QEMU (ARM64)

```bash
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a72 \
    -kernel target/aarch64/debug/wos \
    -nographic
```

## Status

This project is in early development and not yet functional as a full OS.

The goal is to progressively build a minimal kernel and integrate AI-native components.

## Roadmap

- [ ] Minimal bootloader (ARM64)
- [ ] Rust kernel skeleton
- [ ] Memory manager
- [ ] Interrupt handling
- [ ] Wasserstein core module
- [ ] Sinkhorn solver integration
- [ ] QEMU test harness

## License

Private project — all rights reserved.