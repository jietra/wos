// src/arch/aarch64/syscall/syscall.rs

/*
#[link_section = ".user_text"]
fn sys_write(msg: &str) {               // TODO: use &str instead of *const u8 and len: note: beware of the fat pointer
    let ptr = msg.as_ptr() as u64;
    let len = msg.len() as u64;

    unsafe {
        core::arch::asm!(
            "mov x0, {ptr}",
            "mov x1, {len}",
            "mov x8, #0",    // SYS_WRITE
            "svc #0",
            ptr = in(reg) ptr,
            len = in(reg) len,
            out("x0") _, out("x1") _, out("x8") _,
        );
    }
}
*/

#[link_section = ".user_text"]
pub fn sys_write_raw(ptr: *const u8, len: usize) {
    let ptr = ptr as u64;
    let len = len as u64;

    unsafe {
        core::arch::asm!(
            "mov x0, {ptr}",
            "mov x1, {len}",
            "mov x8, #0",    // SYS_WRITE
            "svc #0",
            ptr = in(reg) ptr,
            len = in(reg) len,
            out("x0") _, out("x1") _, out("x8") _,
        );
    }
}