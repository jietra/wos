// src/memory/memory_layout.rs

//! Virtual memory layout for the WOS kernel.
//!
//! This file defines the *virtual* address space organization of the kernel.
//! It does NOT describe physical memory. The MMU maps physical frames into
//! these virtual regions as needed.
//!
//! The goal is to keep the layout stable over time so that future kernel
//! features (scheduler, user space, ELF loader, AI-native modules, etc.)
//! do not require refactoring the entire memory subsystem.

pub mod layout {
    /// Size constants
    pub const GB: usize = 1024 * 1024 * 1024;
    pub const TB: usize = 1024 * GB;

    // -------------------------------------------------------------------------
    // Kernel virtual address space (top of the 48-bit VA range)
    // -------------------------------------------------------------------------
    //
    //  User space:    0x0000_0000_0000_0000 .. 0x0000_FFFF_FFFF_FFFF
    //  Kernel space:  0xFFFF_0000_0000_0000 .. 0xFFFF_FFFF_FFFF_FFFF
    //
    //  We subdivide the kernel space into logical regions:
    //
    //  0xFFFF_FF00_0000_0000  Kernel code/data/bss
    //  0xFFFF_FE00_0000_0000  Kernel heap (dynamic allocations)
    //  0xFFFF_FD00_0000_0000  Device MMIO mappings (UART, GIC, timers, virtio)
    //  0xFFFF_FC00_0000_0000  Reserved (future use)
    //  0xFFFF_FB00_0000_0000  Per-CPU structures
    //  0xFFFF_FA00_0000_0000  Temporary mappings / fixmaps
    //

    /// Kernel base: where `.text`, `.rodata`, `.data`, `.bss` are mapped.
    pub const KERNEL_BASE: usize = 0xFFFF_FF00_0000_0000;

    /// Kernel heap: virtual memory region used for dynamic allocations.
    pub const KERNEL_HEAP_BASE: usize = 0xFFFF_FE00_0000_0000;
    pub const KERNEL_HEAP_SIZE: usize = 1 * TB; // adjustable

    /// Device MMIO region: UART, GIC, timers, virtio, etc.
    pub const DEVICE_BASE: usize = 0x0900_0000;//0xffff000000000000;//0xFFFF_FD00_0000_0000;
    pub const DEVICE_SIZE: usize = 1 * TB;

    /// Reserved region: kept unmapped for future kernel subsystems.
    pub const RESERVED_BASE: usize = 0xFFFF_FC00_0000_0000;
    pub const RESERVED_SIZE: usize = 1 * TB;

    /// Per-CPU structures (stacks, CPU-local data, etc.)
    pub const PERCPU_BASE: usize = 0xFFFF_FB00_0000_0000;
    pub const PERCPU_SIZE: usize = 1 * TB;

    /// Fixmaps: temporary mappings for page-table manipulation, etc.
    pub const FIXMAP_BASE: usize = 0xFFFF_FA00_0000_0000;
    pub const FIXMAP_SIZE: usize = 1 * TB;
}
