// src/arch/riscv64/mmu/stage2.rs

use crate::arch::vm_if::VmArch;

pub struct RiscvVm;

impl VmArch for RiscvVm {
    fn map_page(_ipa: u64, _pa: u64) {}
    fn map_device(_ipa: u64, _pa: u64) {}
    fn get_s2_root() -> u64 { 0 }
    fn get_s2_root_entries() -> *const u64 {
        core::ptr::null()
    }
}
