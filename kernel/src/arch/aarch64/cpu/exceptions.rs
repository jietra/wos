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

// bypassed by scheduler:
/*#[no_mangle]
extern "C" fn irq_current_spx_rust() {
    puts("[IRQ] current EL, SPx\n");
}*/

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

    let ec = (esr >> 26) & 0x3F;

    match ec {
        0x15 => {
            // SVC: we should not normally reach this point, as SVC should be handled by the scheduler or the SVC handler.
            // however, if we reach here, it means that the SVC was not handled properly, and we should print an error message and return.
            puts("[SYNC] SVC (unexpected path)\n");
            return;
        }
        // trap WFI/WFE
        0x01 | 0x00 => {

            /*
            // Debug
            puts("EC = "); put_hex_ln(ec);
            puts("ELR_EL1 = 0x"); put_hex_ln(elr);
            unsafe {
                let instr = *(elr as *const u32);
                puts("INSTR = 0x"); put_hex_ln(instr as u64);
            }
            
            puts("[SYNC] unknown / trapped instruction, halting\n");
            puts("ESR_EL1 = 0x"); put_hex_ln(esr);
            puts("FAR_EL1 = 0x"); put_hex_ln(far);
            puts("ELR_EL1 = 0x"); put_hex_ln(elr);
            
            loop {}
            */
            return;
        }
        _ => {
            puts("[SYNC] from lower EL, AArch64\n");
            puts("ESR_EL1 = 0x"); put_hex_ln(esr);
            puts("FAR_EL1 = 0x"); put_hex_ln(far);
            puts("ELR_EL1 = 0x"); put_hex_ln(elr);

            loop {}
        }
    }
}

#[no_mangle]
extern "C" fn irq_lower_64_rust() {
    puts("[IRQ] from lower EL, AArch64\n");
    loop {}
    // TODO: schedule
    //"let current = CURRENT_PID;
    //"let next = irq_handle_and_schedule(current);
    //"switch_to(next);
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


#[no_mangle]
extern "C" fn log_spsr_elr(spsr: u64, elr: u64) {
    puts("\tRET: SPSR_EL1 = 0x"); put_hex_ln(spsr);
    puts("\tRET: ELR_EL1  = 0x"); put_hex_ln(elr);
}

// TODO: to be removed (only for debug)
#[no_mangle]
extern "C" fn log_spsr_elr_svc(spsr: u64, elr: u64) {
    puts("[SVC] RET: SPSR_EL1 = 0x"); put_hex_ln(spsr);
    puts("[SVC] RET: ELR_EL1  = 0x"); put_hex_ln(elr);
}

// --------
// EL2
// --------
#[no_mangle]
pub extern "C" fn handle_el2_sync_sp0(esr: u64, far: u64) {
    crate::uart_println!("[SYNC SP0] EL2 Fault: ESR={}", esr);
    crate::uart_println!("[SYNC SP0] EL2 Fault: FAR={}", far);

    use crate::arch::mmu::stage2::{S2_ROOT, S2Table};
    
    let esr_real: u64;
    let mut far_real: u64;
    let mut hpfar: u64;
    
    unsafe {
        core::arch::asm!("mrs {0}, esr_el2", out(reg) esr_real);
        core::arch::asm!("mrs {0}, far_el2", out(reg) far_real);
        core::arch::asm!("mrs {0}, hpfar_el2", out(reg) hpfar);
    }
    
    crate::uart_println!("--- S2 FAULT DEBUG ---");
    crate::uart_println!("[SYNC SP0] \tESR_EL2  = 0x{:016x}", esr_real);
    crate::uart_println!("[SYNC SP0] \tFAR_EL2  = 0x{:016x}", far_real);
    crate::uart_println!("[SYNC SP0] \tHPFAR_EL2= 0x{:016x}", hpfar);
    let ec = (esr_real >> 26) & 0x3F;
    crate::uart_println!("[SYNC SP0] \tEC       = 0x{:02x}", ec);

    // IPA = HPFAR_EL2[47:12] << 12
    let ipa = (hpfar & ((1 << 48) - 1)) << 8; // HPFAR_EL2 bits [47:4] → IPA[47:12]
    crate::uart_println!("[SYNC SP0] \tIPA      = 0x{:016x}", ipa);

    // indices 3 niveaux
    let i1 = ((ipa >> 30) & 0x1FF) as usize;
    let i2 = ((ipa >> 21) & 0x1FF) as usize;
    let i3 = ((ipa >> 12) & 0x1FF) as usize;

    crate::uart_println!("[SYNC SP0] \ti1={}, i2={}, i3={}", i1, i2, i3);

    let l1 = unsafe { S2_ROOT.entries[i1] };
    crate::uart_println!("[SYNC SP0] \tS2_ROOT[{}] = 0x{:016x}", i1, l1);

    if l1 & 0b11 == 0b11 {
        let l2_pa = l1 & !0xFFF;
        let l2 = l2_pa as *const u64;
        let l2e = unsafe { *l2.add(i2) };
        crate::uart_println!("[SYNC SP0] \tL2[{}] @ 0x{:016x} = 0x{:016x}", i2, l2_pa, l2e);

        if l2e & 0b11 == 0b11 {
            let l3_pa = l2e & !0xFFF;
            let l3 = l3_pa as *const u64;
            let l3e = unsafe { *l3.add(i3) };
            crate::uart_println!("[SYNC SP0] \tL3[{}] @ 0x{:016x} = 0x{:016x}", i3, l3_pa, l3e);
        } else {
            crate::uart_println!("[SYNC SP0] \tL2[{}] is not a table/page descriptor", i2);
        }
    } else {
        crate::uart_println!("[SYNC SP0] \tS2_ROOT[{}] is not a table descriptor", i1);
    }

    let mut ttbr0: u64;
    let mut tcr1: u64;
    unsafe {
        core::arch::asm!("mrs {0}, ttbr0_el1", out(reg) ttbr0);
        core::arch::asm!("mrs {0}, tcr_el1", out(reg) tcr1);
    }
    crate::uart_println!("TTBR0_EL1 = 0x{:016x}", ttbr0);
    crate::uart_println!("TCR_EL1   = 0x{:016x}", tcr1);


    crate::uart_println!("--- END S2 FAULT DEBUG ---");

    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_irq_sp0(esr: u64, far: u64) {
    crate::uart_println!("[IRQ SP0] ESR={}", esr);
    crate::uart_println!("[IRQ SP0] FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_fiq_sp0(esr: u64, far: u64) {
    crate::uart_println!("[FIQ SP0] ESR={}", esr);
    crate::uart_println!("[FIQ SP0] FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_serror_sp0(esr: u64, far: u64) {
    crate::uart_println!("[SERROR SP0] ESR={}", esr);
    crate::uart_println!("[SERROR SP0] FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_sync(esr: u64, far: u64) {
    crate::uart_println!("[SYNC] EL2 Fault: ESR={}", esr);
    crate::uart_println!("[SYNC] EL2 Fault: FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_irq(esr: u64, far: u64) {
    crate::uart_println!("[IRQ] ESR={}", esr);
    crate::uart_println!("[IRQ] FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_fiq(esr: u64, far: u64) {
    crate::uart_println!("[FIQ] ESR={}", esr);
    crate::uart_println!("[FIQ] FAR={}", far);
    loop {};
}

#[no_mangle]
pub extern "C" fn handle_el2_serror(esr: u64, far: u64) {
    crate::uart_println!("[SERROR] ESR={}", esr);
    crate::uart_println!("[SERROR] FAR={}", far);
    loop {};
}