// src/tasks.rs

#[no_mangle]
pub extern "C" fn task0_entry() -> ! {
    let mut i = 0u64;
    loop {
        crate::uart_println!("[task0] running, i={}", i);
        i += 1;
        for _ in 0..1_000_000 { unsafe { core::arch::asm!("nop") } }
    }
}

#[no_mangle]
pub extern "C" fn task1_entry() -> ! {
    let mut i = 0u64;
    loop {
        crate::uart_println!("[task1] running, i={}", i);
        i += 1;
        for _ in 0..1_000_000 { unsafe { core::arch::asm!("nop") } }
    }
}

#[no_mangle]
pub extern "C" fn task2_entry() -> ! {
    let mut i = 0u64;
    loop {
        crate::uart_println!("[task2] running, i={}", i);
        i += 1;
        for _ in 0..1_000_000 { unsafe { core::arch::asm!("nop") } }
    }
}