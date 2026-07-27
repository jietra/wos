// src/arch/riscv64/syscall/syscall.rs

#[link_section = ".user_text"]
pub fn sys_write_raw(ptr: *const u8, len: usize) {
    unsafe {
        core::arch::asm!(
            "mv a0, {ptr}",
            "mv a1, {len}",
            "li a7, 0",
            "ecall",
            ptr = in(reg) ptr as u64,
            len = in(reg) len as u64,
            out("a0") _, out("a1") _, out("a7") _,
        );
    }
}
