// src/arch/vm_if.rs

pub trait VmArch {
    fn map_page(ipa: u64, pa: u64);
    fn map_device(ipa: u64, pa: u64);
    fn get_s2_root() -> u64;
    fn get_s2_root_entries() -> *const u64;
}
