// src/arch/aarch64/cpu/exceptions.rs

use crate::drivers::uart::puts;
use crate::utils::print::put_hex_ln;

// -----------------------------------------------------------------------------

extern "C" {
    static exception_vectors: u8;
}

#[no_mangle]
extern "C" fn sync_current_sp0_rust() {
    puts("[SYNC] current EL, SP0\n");
}

#[no_mangle]
extern "C" fn irq_current_sp0_rust() {
    puts("[IRQ] current EL, SP0\n");
}

#[no_mangle]
extern "C" fn fiq_current_sp0_rust() {
    puts("[FIQ] current EL, SP0\n");
}

#[no_mangle]
extern "C" fn serr_current_sp0_rust() {
    puts("[SERR] current EL, SP0\n");
}

// -----------------------------------------------------------------------------

#[no_mangle]
extern "C" fn sync_current_spx_rust() {
    let esr: u64;
    let far: u64;
    let elr: u64;

    unsafe {
        core::arch::asm!(
            "mrs {0}, ESR_EL1",
            "mrs {1}, FAR_EL1",
            "mrs {2}, ELR_EL1",
            out(reg) esr,
            out(reg) far,
            out(reg) elr,
            options(nostack, preserves_flags),
        );
    }

    puts("[SYNC] Exception!\n");
    puts("ESR_EL1 = 0x"); put_hex_ln(esr);
    puts("FAR_EL1 = 0x"); put_hex_ln(far);
    puts("ELR_EL1 = 0x"); put_hex_ln(elr);

    loop {} // Otherwise, eret might return to same instruction and cause infinite exceptions
}

#[no_mangle]
extern "C" fn irq_current_spx_rust() {
    puts("[IRQ] current EL, SPx\n");
    crate::arch::aarch64::irq::handler::handle_irq();   // handle IRQ
}

#[no_mangle]
extern "C" fn fiq_current_spx_rust() {
    puts("[FIQ] current EL, SPx\n");
}

#[no_mangle]
extern "C" fn serr_current_spx_rust() {
    puts("[SERR] current EL, SPx\n");
}

// -----------------------------------------------------------------------------

#[no_mangle]
extern "C" fn sync_lower_64_rust() {
    puts("[SYNC] from lower EL, AArch64\n");
}

#[no_mangle]
extern "C" fn irq_lower_64_rust() {
    puts("[IRQ] from lower EL, AArch64\n");
}

#[no_mangle]
extern "C" fn fiq_lower_64_rust() {
    puts("[FIQ] from lower EL, AArch64\n");
}

#[no_mangle]
extern "C" fn serr_lower_64_rust() {
    puts("[SERR] from lower EL, AArch64\n");
}

// -----------------------------------------------------------------------------

#[no_mangle]
extern "C" fn sync_lower_32_rust() {
    puts("[SYNC] from lower EL, AArch32\n");
}

#[no_mangle]
extern "C" fn irq_lower_32_rust() {
    puts("[IRQ] from lower EL, AArch32\n");
}

#[no_mangle]
extern "C" fn fiq_lower_32_rust() {
    puts("[FIQ] from lower EL, AArch32\n");
}

#[no_mangle]
extern "C" fn serr_lower_32_rust() {
    puts("[SERR] from lower EL, AArch32\n");
}

// -----------------------------------------------------------------------------
// Exception vector initialization (sets VBAR_EL1 to point to our exception vectors)
// -----------------------------------------------------------------------------
pub unsafe fn init_exceptions() {

    let addr = &exception_vectors as *const _ as u64 ;
    puts("\tException vect \t= 0x"); put_hex_ln(addr);

    // --- Set VBAR_EL1 to point to our exception vectors and synchronize the instruction stream ---
    core::arch::asm!(
        "msr VBAR_EL1, {0}",
        in(reg) addr,
        options(nostack, preserves_flags),
    ); // Put the address of our exception vectors in VBAR_EL1
    core::arch::asm!("isb"); // Synchronize the instruction stream to ensure the new VBAR_EL1 value is used immediately
    
    
    // --- Read VBAR_EL1 to confirm it's correctly set to the address of our exception vectors
    let vbar: u64;
    core::arch::asm!("mrs {0}, VBAR_EL1", out(reg) vbar);
    puts("\tVBAR_EL1 \t= 0x"); put_hex_ln(vbar);
}