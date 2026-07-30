// src/arch/mod.rs

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

pub use self::arch_impl::*;

#[cfg(target_arch = "aarch64")]
mod arch_impl {
    pub use super::aarch64::*;
}

#[cfg(target_arch = "riscv64")]
mod arch_impl {
    pub use super::riscv64::*;
}

pub mod vm_if;

#[cfg(target_arch = "aarch64")]
pub type ArchVm = aarch64::mmu::stage2::AArch64Vm;

#[cfg(target_arch = "riscv64")]
pub type ArchVm = riscv64::mmu::stage2::RiscvVm;
