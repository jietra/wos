use crate::drivers::uart::puts;

/*
// Declare the exception vectors symbol (defined in assembly) so Rust can reference it if needed
#[allow(dead_code)]
extern "C" {
    static exception_vectors: u8;
}*/

#[no_mangle]
extern "C" fn irq_current_spx_rust() {
    puts("[IRQ] current EL, SPx\n");
}

#[no_mangle]
extern "C" fn sync_current_spx_rust() {
    puts("[SYNC] current EL, SPx\n");
}

#[no_mangle]
extern "C" fn irq_current_sp0_rust() {
    puts("[IRQ] current EL, SP0\n");
}

#[no_mangle]
extern "C" fn sync_current_sp0_rust() {
    puts("[SYNC] current EL, SP0\n");
}

// -----------------------------------------------------------------------------
// Exception vector initialization (sets VBAR_EL1 to point to our exception vectors)
// -----------------------------------------------------------------------------
pub unsafe fn init_exceptions() {
    extern "C" {
        static exception_vectors: u8;
    }

    let addr = unsafe { &exception_vectors as *const _ as u64 };

    unsafe {
        core::arch::asm!(
            "msr VBAR_EL1, {0}",
            in(reg) addr,
            options(nostack, preserves_flags),
        );
    }
}