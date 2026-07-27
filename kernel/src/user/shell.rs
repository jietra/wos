// src/user/shell.rs

use crate::arch::syscall::syscall::sys_write_raw;

#[link_section = ".user_text"]
#[no_mangle]
pub extern "C" fn user_shell_entry() -> ! {
    sys_write_raw(HELLO.as_ptr(), HELLO.len());
    loop {
        //unsafe { core::arch::asm!("wfi"); } // Wait For Interrupt
    }
}

#[link_section = ".user_data"]
#[no_mangle]
pub static HELLO: [u8; N] = *b"
--------------------------------------
|      Hello from EL0 userland!      |
--------------------------------------
\n\n";

const N: usize = b"
--------------------------------------
|      Hello from EL0 userland!      |
--------------------------------------
\n\n".len();