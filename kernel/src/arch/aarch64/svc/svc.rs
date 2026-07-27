// src/arch/aarch64/svc/svc.rs

#[no_mangle]
pub extern "C" fn svc_dispatch(num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {


    crate::uart_println!("[SVC] num={} arg0=0x{:016x} arg1={}", num, arg0, arg1);


    match num {
        0 => { // SYS_WRITE
            let ptr = arg0 as *const u8;
            let len = arg1 as usize;

            let buf = unsafe { core::slice::from_raw_parts(ptr, len) };
            let s = unsafe { core::str::from_utf8_unchecked(buf) };

            crate::drivers::uart::puts(s);
            0
        }
        _ => {
            crate::drivers::uart::puts("[SVC] Unknown syscall\n");
            u64::MAX
        }
    }
}
