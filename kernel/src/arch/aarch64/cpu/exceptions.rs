// src/arch/aarch64/cpu/exceptions.rs

use crate::drivers::uart::puts;
use crate::utils::print::put_hex_ln;
use crate::config::{GUEST_UART_IPA, GUEST_GICD_IPA, GUEST_GICC_IPA, GUEST_VIRTIO_BLK_IPA, VIRTIO_MMIO_BASE};
use crate::arch::aarch64::uart::pl011::VIRT_UART;
use crate::arch::aarch64::gic::gicv2_virt::{VIRT_GICD, VIRT_GICC};
use crate::arch::aarch64::virtio::blk_virt::VIRTIO_BLK;
use crate::arch::mmu::stage2::S2_ROOT;
//use crate::arch::aarch64::mmu::stage2::map_ipa_page_device;
use crate::drivers::uart::putc;
use crate::drivers::pci::device::{pci_mmio_read, pci_mmio_write};
use crate::drivers::pci::host::{ecam_read, ecam_write};

use crate::arch::vm_if::VmArch;
use crate::arch::ArchVm;

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

    loop {}
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
    //crate::uart_println!("sync_lower_64_rust - svc");
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

    //crate::uart_println!("ec = {}", ec);

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
    crate::uart_println!("\n[SYNC SP0] === EL2 Exception: ESR=0x{:016x}, FAR=0x{:016x}", esr, far);

    let esr_real: u64;
    let far_real: u64;
    let hpfar: u64;
    
    unsafe {
        core::arch::asm!("mrs {0}, esr_el2", out(reg) esr_real);
        core::arch::asm!("mrs {0}, far_el2", out(reg) far_real);
        core::arch::asm!("mrs {0}, hpfar_el2", out(reg) hpfar);
    }

    let ec = (esr_real >> 26) & 0x3F;
    let iss = esr_real & 0xFFFFFF;

    // Check if the exception is a Stage 2 translation fault (EC=0x24)
    if ec != 0x24 {
        crate::uart_println!("[SYNC SP0] EL2 Fault: EC =0x{:x}", ec);
        crate::uart_println!("[SYNC SP0] EL2 Fault: FAR=0x{:016x}", far);
        loop {}
    }
    
    // IPA = HPFAR_EL2[47:12] << 12 (S2 fault)
    // IPA[47:12] = HPFAR_EL2[47:4]
    // IPA[11:0]  = FAR_EL2[11:0]
    let ipa_page = (hpfar & ((1 << 48) - 1)) << 8; // HPFAR_EL2[47:4] -> IPA[47:12]
    let ipa      = ipa_page | (far_real & 0xFFF);  // add intra-page offset

    /*
    // debug: 3 level indices
        let i1 = ((ipa >> 30) & 0x1FF) as usize;
        let i2 = ((ipa >> 21) & 0x1FF) as usize;
        let i3 = ((ipa >> 12) & 0x1FF) as usize;

        crate::uart_println!("\t[SYNC SP0] \ti1={}, i2={}, i3={}", i1, i2, i3);

        let l1 = unsafe { S2_ROOT.entries[i1] };
        crate::uart_println!("\t[SYNC SP0] \tS2_ROOT[{}] = 0x{:016x}", i1, l1);

        if l1 & 0b11 == 0b11 {
            let l2_pa = l1 & !0xFFF;
            let l2 = l2_pa as *const u64;
            let l2e = unsafe { *l2.add(i2) };
            crate::uart_println!("\t[SYNC SP0] \tL2[{}] @ 0x{:016x} = 0x{:016x}", i2, l2_pa, l2e);

            if l2e & 0b11 == 0b11 {
                let l3_pa = l2e & !0xFFF;
                let l3 = l3_pa as *const u64;
                let l3e = unsafe { *l3.add(i3) };
                crate::uart_println!("\t[SYNC SP0] \tL3[{}] @ 0x{:016x} = 0x{:016x}", i3, l3_pa, l3e);
            } else {
                crate::uart_println!("\t[SYNC SP0] \tL2[{}] is not a table/page descriptor", i2);
            }
        } else {
            crate::uart_println!("\t[SYNC SP0] \tS2_ROOT[{}] is not a table descriptor", i1);
        }
    // end debug
    */

    // bit 6 of ISS: 0 = read, 1 = write
    let is_write = ((iss >> 6) & 1) == 1;
    let rt = (iss & 0x1F) as u8;

    // xRt value for writes
    let mut value: u64 = 0;
    if is_write {
        unsafe {
            match rt {
                0  => core::arch::asm!("mov {0}, x0",  out(reg) value),
                1  => core::arch::asm!("mov {0}, x1",  out(reg) value),
                2  => core::arch::asm!("mov {0}, x2",  out(reg) value),
                3  => core::arch::asm!("mov {0}, x3",  out(reg) value),
                4  => core::arch::asm!("mov {0}, x4",  out(reg) value),
                5  => core::arch::asm!("mov {0}, x5",  out(reg) value),
                6  => core::arch::asm!("mov {0}, x6",  out(reg) value),
                7  => core::arch::asm!("mov {0}, x7",  out(reg) value),
                8  => core::arch::asm!("mov {0}, x8",  out(reg) value),
                9  => core::arch::asm!("mov {0}, x9",  out(reg) value),
                10 => core::arch::asm!("mov {0}, x10", out(reg) value),
                11 => core::arch::asm!("mov {0}, x11", out(reg) value),
                12 => core::arch::asm!("mov {0}, x12", out(reg) value),
                13 => core::arch::asm!("mov {0}, x13", out(reg) value),
                14 => core::arch::asm!("mov {0}, x14", out(reg) value),
                15 => core::arch::asm!("mov {0}, x15", out(reg) value),
                16 => core::arch::asm!("mov {0}, x16", out(reg) value),
                17 => core::arch::asm!("mov {0}, x17", out(reg) value),
                18 => core::arch::asm!("mov {0}, x18", out(reg) value),
                19 => core::arch::asm!("mov {0}, x19", out(reg) value),
                20 => core::arch::asm!("mov {0}, x20", out(reg) value),
                21 => core::arch::asm!("mov {0}, x21", out(reg) value),
                22 => core::arch::asm!("mov {0}, x22", out(reg) value),
                23 => core::arch::asm!("mov {0}, x23", out(reg) value),
                24 => core::arch::asm!("mov {0}, x24", out(reg) value),
                25 => core::arch::asm!("mov {0}, x25", out(reg) value),
                26 => core::arch::asm!("mov {0}, x26", out(reg) value),
                27 => core::arch::asm!("mov {0}, x27", out(reg) value),
                28 => core::arch::asm!("mov {0}, x28", out(reg) value),
                29 => core::arch::asm!("mov {0}, x29", out(reg) value),
                30 => core::arch::asm!("mov {0}, x30", out(reg) value),
                _  => {}
            }
        }
    }

    let wval = value as u32;
    let sas = ((iss >> 22) & 0b11) as u8;//(iss & 0b11) as u8;
    let size = match sas {
        0 => 1,   // byte
        1 => 2,   // halfword
        2 => 4,   // word
        3 => 8,   // doubleword
        _ => 4,
    };

    // Dispatch virtual MMIO
    if ipa >= 0x1000_0000 && ipa < 0x1100_0000 {
        if is_write {
            ecam_write(ipa, wval);
        } else {
            let val = ecam_read(ipa, size);
            unsafe {write_xrt(rt, val as u64)};
        }
    } else if ipa >= 0x70000000 && ipa < 0x70000000 + 0x1000 {
        unsafe { handle_virtio_blk_pci(ipa, is_write, wval, rt, size) };
    } else if ipa >= VIRTIO_MMIO_BASE && ipa < VIRTIO_MMIO_BASE + 0x1000 {
        handle_virtio_mmio(ipa, is_write, wval, rt);
    } else if ipa >= GUEST_UART_IPA && ipa < GUEST_UART_IPA + 0x1000 {
        let offset = ipa - 0x0900_0000;
        unsafe {
            if is_write {
                VIRT_UART.pl011_mmio_write(offset, wval as u32);
            } else {
                let val = VIRT_UART.pl011_mmio_read(offset);
                write_xrt(rt, val as u64);
            }
        } 
        //handle_uart_mmio(ipa, is_write, wval, rt);
    } else if ipa >= GUEST_GICD_IPA && ipa < GUEST_GICD_IPA + 0x1000 {
        //handle_gicd_mmio(ipa, is_write, wval, rt);
    } else if ipa >= GUEST_GICC_IPA && ipa < GUEST_GICC_IPA + 0x1000 {
        //handle_gicc_mmio(ipa, is_write, wval, rt);
    } else if ipa >= GUEST_VIRTIO_BLK_IPA && ipa < GUEST_VIRTIO_BLK_IPA + 0x1000 {
        //handle_virtio_blk_mmio(ipa, is_write, wval, rt);
    } else {
        crate::uart_println!("[EL2] MMIO unknown IPA=0x{:016x}", ipa);
        crate::uart_println!("[EL2] ESR_EL2=0x{:016x}", esr);
        crate::uart_println!("[EL2] FAR_EL2=0x{:016x}", far);
        crate::uart_println!("--- S2 FAULT DEBUG ---");
        crate::uart_println!("[SYNC SP0] \tESR_EL2  = 0x{:016x}", esr_real);
        crate::uart_println!("[SYNC SP0] \tFAR_EL2  = 0x{:016x}", far_real);
        crate::uart_println!("[SYNC SP0] \tHPFAR_EL2= 0x{:016x}", hpfar);

        crate::uart_println!("[SYNC SP0] \tEC       = 0x{:02x}", ec);
        crate::uart_println!("[SYNC SP0] \tIPA      = 0x{:016x}", ipa);

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

    // advance ELR_EL2
    unsafe { advance_elr(); }
}

unsafe fn handle_virtio_blk_pci(ipa: u64, is_write: bool, value: u32, rt: u8, size: u8) {
    if is_write {
        crate::uart_println!("[EL2] VIRTIO_BLK MMIO WRITE IPA=0x{:016x}, value=0x{:08x}, rt=x{:02x}", ipa, value, rt);
        pci_mmio_write(ipa, size, value as u64);
    } else {
        let read_value: u32 = pci_mmio_read(ipa, size);
        crate::uart_println!("[EL2] VIRTIO_BLK MMIO READ IPA=0x{:016x}, value=0x{:08x}, rt=x{:02x}", ipa, read_value, rt);
        write_xrt(rt, read_value as u64);
    }
}

unsafe fn write_xrt(rt: u8, value: u64) {
    //crate::uart_println!("[ECAM] will write 0x{:08x} into x{}", value, rt);
    let v32 = value as u32;
    macro_rules! wr {
        ($reg:literal) => {
            core::arch::asm!(
                concat!("mov w", $reg, ", {tmp:w}"),
                tmp = in(reg) v32,
            )
        };
    }

    match rt {
        0  => wr!("0"),
        1  => wr!("1"),
        2  => wr!("2"),
        3  => wr!("3"),
        4  => wr!("4"),
        5  => wr!("5"),
        6  => wr!("6"),
        7  => wr!("7"),
        8  => wr!("8"),
        9  => wr!("9"),
        10 => wr!("10"),
        11 => wr!("11"),
        12 => wr!("12"),
        13 => wr!("13"),
        14 => wr!("14"),
        15 => wr!("15"),
        16 => wr!("16"),
        17 => wr!("17"),
        18 => wr!("18"),
        19 => wr!("19"),
        20 => wr!("20"),
        21 => wr!("21"),
        22 => wr!("22"),
        23 => wr!("23"),
        24 => wr!("24"),
        25 => wr!("25"),
        26 => wr!("26"),
        27 => wr!("27"),
        28 => wr!("28"),
        29 => wr!("29"),
        30 => wr!("30"),
        _ => {}
    }
}

unsafe fn read_xrt(rt: u64) -> u64 {
    let mut value: u64 = 0;
    match rt {
        0  => core::arch::asm!("mov {0}, x0", out(reg) value),
        1  => core::arch::asm!("mov {0}, x1", out(reg) value),
        2  => core::arch::asm!("mov {0}, x2", out(reg) value),
        3  => core::arch::asm!("mov {0}, x3", out(reg) value),
        4  => core::arch::asm!("mov {0}, x4", out(reg) value),
        5  => core::arch::asm!("mov {0}, x5", out(reg) value),
        6  => core::arch::asm!("mov {0}, x6", out(reg) value),
        7  => core::arch::asm!("mov {0}, x7", out(reg) value),
        8  => core::arch::asm!("mov {0}, x8", out(reg) value),
        9  => core::arch::asm!("mov {0}, x9", out(reg) value),
        10 => core::arch::asm!("mov {0}, x10", out(reg) value),
        11 => core::arch::asm!("mov {0}, x11", out(reg) value),
        12 => core::arch::asm!("mov {0}, x12", out(reg) value),
        13 => core::arch::asm!("mov {0}, x13", out(reg) value),
        14 => core::arch::asm!("mov {0}, x14", out(reg) value),
        15 => core::arch::asm!("mov {0}, x15", out(reg) value),
        16 => core::arch::asm!("mov {0}, x16", out(reg) value),
        17 => core::arch::asm!("mov {0}, x17", out(reg) value),
        18 => core::arch::asm!("mov {0}, x18", out(reg) value),
        19 => core::arch::asm!("mov {0}, x19", out(reg) value),
        20 => core::arch::asm!("mov {0}, x20", out(reg) value),
        21 => core::arch::asm!("mov {0}, x21", out(reg) value),
        22 => core::arch::asm!("mov {0}, x22", out(reg) value),
        23 => core::arch::asm!("mov {0}, x23", out(reg) value),
        24 => core::arch::asm!("mov {0}, x24", out(reg) value),
        25 => core::arch::asm!("mov {0}, x25", out(reg) value),
        26 => core::arch::asm!("mov {0}, x26", out(reg) value),
        27 => core::arch::asm!("mov {0}, x27", out(reg) value),
        28 => core::arch::asm!("mov {0}, x28", out(reg) value),
        29 => core::arch::asm!("mov {0}, x29", out(reg) value),
        30 => core::arch::asm!("mov {0}, x30", out(reg) value),
        _  => {}
    }
    value
}

unsafe fn advance_elr() {
    let mut elr: u64;
    core::arch::asm!("mrs {0}, elr_el2", out(reg) elr);
    elr += 4; // assuming 4-byte instruction
    core::arch::asm!("msr elr_el2, {0}", in(reg) elr);
}

#[repr(C)]
struct VirtqDesc {
    addr: u64,   // adresse guest du buffer
    len: u32,    // taille
    flags: u16,  // NEXT, WRITE, etc.
    next: u16,   // index du prochain
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; 256], // indices de descripteurs
    // event (optionnel)
}

#[repr(C)]
struct VirtqUsedElem {
    id: u32,   // index du descripteur
    len: u32,  // taille consommée
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; 256],
    // event (optionnel)
}

static mut QUEUE_SEL: u16 = 0;
static mut QUEUE_NUM: u16 = 0;
static mut QUEUE_READY: u16 = 0;

static mut QUEUE_DESC_PA: u64 = 0;
static mut QUEUE_AVAIL_PA: u64 = 0;
static mut QUEUE_USED_PA: u64 = 0;

static mut DEVICE_STATUS: u32 = 0;
use core::cell::UnsafeCell;
//static DEVICE_STATUS: UnsafeCell<u32> = UnsafeCell::new(0);
pub struct VirtioState {
    pub device_status: u32,
    pub queue_ready: u16,
    pub queue_sel: u16,
    pub queue_num: u16,
}

pub struct VirtioCell(UnsafeCell<VirtioState>);

// Explicitly tell Rust you ensure safety
unsafe impl Sync for VirtioCell {}

static VIRTIO: VirtioCell = VirtioCell(UnsafeCell::new(VirtioState {
    device_status: 0,
    queue_ready: 0,
    queue_sel: 0,
    queue_num: 0,
}));

fn virtio_notify(queue_index: u64) {
    if queue_index != 0 { return; }

    unsafe {
        if QUEUE_READY == 0 { return; }

        let avail = QUEUE_AVAIL_PA as *mut VirtqAvail;
        let desc  = QUEUE_DESC_PA  as *mut VirtqDesc;
        let used  = QUEUE_USED_PA  as *mut VirtqUsed;

        let avail_idx = (*avail).idx;
        static mut LAST_AVAIL_IDX: u16 = 0;

        while LAST_AVAIL_IDX != avail_idx {
            let ring_idx = LAST_AVAIL_IDX as usize % QUEUE_NUM as usize;
            let desc_idx = (*avail).ring[ring_idx] as usize;

            let d = &*desc.add(desc_idx);

            // Read guest buffer
            let buf = core::slice::from_raw_parts(d.addr as *const u8, d.len as usize);

            // Print char
            for &b in buf {
                putc(b);
            }

            // Mark as "used"
            let used_idx = (*used).idx as usize % QUEUE_NUM as usize;
            (*used).ring[used_idx] = VirtqUsedElem {
                id: desc_idx as u32,
                len: d.len,
            };
            (*used).idx += 1;

            LAST_AVAIL_IDX += 1;
        }
    }
}

fn handle_virtio_mmio(ipa: u64, is_write: bool, _val: u32, rt: u8) {
    crate::uart_println!("[VIRTIO] MMIO ipa=0x{:016x}, write={:01x}, off=0x{:04x}, rt={:02x}", ipa, is_write, ipa - VIRTIO_MMIO_BASE, rt);
    //if ipa < VIRTIO_MMIO_BASE || ipa >= VIRTIO_MMIO_BASE + 0x1000 {
    //    return false; // pas virtio-mmio
    //}

    let off = ipa - VIRTIO_MMIO_BASE;
    let mut ret: u64 = 0;

    if !is_write {
        crate::uart_println!("[VIRTIO] MMIO read off=0x{:04x}, rt={:02x}", off, rt);
        ret = match off {
            0x000 => 0x74726976, // MAGIC "virt"
            0x004 => 2,          // VERSION
            0x008 => 3,          // DEVICE_ID = console
            0x00C => 0x1234,     // VENDOR_ID

            0x010 => 0,          // DEVICE_FEATURES
            0x034 => 256,        // QUEUE_NUM_MAX
            0x044 => unsafe { QUEUE_READY as u64 },         // QUEUE_READY
            0x060 => 0,                                     // INTERRUPT_STATUS
            0x070 => unsafe { DEVICE_STATUS as u64 },       // STATUS

            _ => 0,
        };

        crate::uart_println!("[VIRTIO] write_xrt(rt={:02x}, ret=0x{:08x})", rt, ret);
        unsafe { write_xrt(rt, ret); }

        unsafe {
            let mut check: u64 = 0;
            core::arch::asm!("mov {0}, x6", out(reg) check);
            crate::uart_println!("[VIRTIO] after write_xrt: x6 = 0x{:08x}", check as u32);
        }

        return;
    } else {
        // Write: store values for later 
        crate::uart_println!("[VIRTIO] MMIO read_xrt or virtio_notify");
        unsafe {
            match off {
                0x014 => { /* DEVICE_FEATURES_SEL: ignore          */ }
                0x020 => { /* DRIVER_FEATURES:     ignore or store */ }
                0x024 => { /* DRIVER_FEATURES_SEL: ignore          */ }
                
                0x030 => QUEUE_SEL     = read_xrt(rt as u64) as u16,    // QUEUE_SEL
                0x038 => QUEUE_NUM     = read_xrt(rt as u64) as u16,    // QUEUE_NUM
                0x044 => QUEUE_READY   = read_xrt(rt as u64) as u16,    // QUEUE_READY
                0x050 => virtio_notify( QUEUE_SEL as u64 ),             // QUEUE_NOTIFY
                0x070 => DEVICE_STATUS = read_xrt(rt as u64) as u32,    // STATUS

                _ => {}
            }
        }
}
}

/*
pub fn handle_uart_mmio(ipa: u64, is_write: bool, val: u32, rt: u8) {
    use crate::config::GUEST_UART_IPA;
    use crate::arch::aarch64::uart::pl011::VIRT_UART;

    let offset = (ipa - GUEST_UART_IPA) as u32;
    crate::uart_println!("[EL2] ipa=0x{:016x}, offset=0x{:x}, is_write={:01x}, rt={:02x}", ipa, offset, is_write, rt);

    unsafe {
        if is_write {
            match offset {
                0x00 => {
                    // UARTDR
                    let ch = (val & 0xFF) as u8;
                    crate::uart_println!("{}", ch as char);
                }
                _ => {
                    crate::uart_println!("[UART] write offset=0x{:x}, val=0x{:x}", offset, val);
                }
            }
        } else {
            // read path
            let mut ret: u32 = 0;
            match offset {
                0x18 => ret = 0x80 | 0x10 | 0x01,   // TXFE | RXFE | CTS, // UARTFR : TX FIFO empty + RX FIFO empty          // FR : TXFE=1
                0x30 => ret = VIRT_UART.cr,         // CR : return the current control register value (what the guest wrote)
                0x2C => ret = VIRT_UART.lcrh,
                0x24 => ret = VIRT_UART.ibrd,
                0x28 => ret = VIRT_UART.fbrd,
                _ => {
                    crate::uart_println!("[UART] read offset=0x{:x}", offset);
                    ret = 0;
                }
            }

            // write ret in xRt
            match rt {
                0  => core::arch::asm!("mov x0, {0}", in(reg) ret as u64),
                1  => core::arch::asm!("mov x1, {0}", in(reg) ret as u64),
                2  => core::arch::asm!("mov x2, {0}", in(reg) ret as u64),
                3  => core::arch::asm!("mov x3, {0}", in(reg) ret as u64),
                4  => core::arch::asm!("mov x4, {0}", in(reg) ret as u64),
                5  => core::arch::asm!("mov x5, {0}", in(reg) ret as u64),
                6  => core::arch::asm!("mov x6, {0}", in(reg) ret as u64),
                7  => core::arch::asm!("mov x7, {0}", in(reg) ret as u64),
                8  => core::arch::asm!("mov x8, {0}", in(reg) ret as u64),
                9  => core::arch::asm!("mov x9, {0}", in(reg) ret as u64),
                10 => core::arch::asm!("mov x10, {0}", in(reg) ret as u64),
                11 => core::arch::asm!("mov x11, {0}", in(reg) ret as u64),
                12 => core::arch::asm!("mov x12, {0}", in(reg) ret as u64),
                13 => core::arch::asm!("mov x13, {0}", in(reg) ret as u64),
                14 => core::arch::asm!("mov x14, {0}", in(reg) ret as u64),
                15 => core::arch::asm!("mov x15, {0}", in(reg) ret as u64),
                16 => core::arch::asm!("mov x16, {0}", in(reg) ret as u64),
                17 => core::arch::asm!("mov x17, {0}", in(reg) ret as u64),
                18 => core::arch::asm!("mov x18, {0}", in(reg) ret as u64),
                19 => core::arch::asm!("mov x19, {0}", in(reg) ret as u64),
                20 => core::arch::asm!("mov x20, {0}", in(reg) ret as u64),
                21 => core::arch::asm!("mov x21, {0}", in(reg) ret as u64),
                22 => core::arch::asm!("mov x22, {0}", in(reg) ret as u64),
                23 => core::arch::asm!("mov x23, {0}", in(reg) ret as u64),
                24 => core::arch::asm!("mov x24, {0}", in(reg) ret as u64),
                25 => core::arch::asm!("mov x25, {0}", in(reg) ret as u64),
                26 => core::arch::asm!("mov x26, {0}", in(reg) ret as u64),
                27 => core::arch::asm!("mov x27, {0}", in(reg) ret as u64),
                28 => core::arch::asm!("mov x28, {0}", in(reg) ret as u64),
                29 => core::arch::asm!("mov x29, {0}", in(reg) ret as u64),
                30 => core::arch::asm!("mov x30, {0}", in(reg) ret as u64),
                // ... etc
                _  => {}
            }
        }
        /*match (offset, is_write) {
            (0x00, true) => {
                VIRT_UART.dr = val;
                let ch = (val & 0xFF) as u8;
                crate::uart_println!("{}", ch as char);
            }
            (0x18, true) => {
                VIRT_UART.fr = val;
            }
            (0x30, true) => {
                VIRT_UART.cr = val;
            }
            (0x2C, true) => {
                VIRT_UART.lcrh = val;
            }
            (0x24, true) => {
                VIRT_UART.ibrd = val;
            }
            (0x28, true) => {
                VIRT_UART.fbrd = val;
            }
            _ => {
                crate::uart_println!("[UART] MMIO offset=0x{:x}, write={:01x}, val=0x{:x}", offset, is_write, val);
            }
        }
        */
    }
}


pub fn handle_gicd_mmio(ipa: u64, is_write: bool, val: u32, _rt: u8) {
    let offset = (ipa - GUEST_GICD_IPA) as u32;

    unsafe {
        match (offset, is_write) {
            // GICD_CTLR
            (0x000, true) => {
                VIRT_GICD.ctlr = val;
            }
            // GICD_ISENABLERn (0x100 + 4*n)
            (off, true) if off >= 0x100 && off < 0x180 => {
                let n = ((off - 0x100) / 4) as usize;
                VIRT_GICD.isenabler[n] = val;
            }
            // GICD_ICENABLERn
            (off, true) if off >= 0x180 && off < 0x200 => {
                let n = ((off - 0x180) / 4) as usize;
                VIRT_GICD.icenabler[n] = val;
            }
            // GICD_IPRIORITYRn (0x400 + 4*n)
            (off, true) if off >= 0x400 && off < 0x800 => {
                let n = ((off - 0x400) / 4) as usize;
                let base = n * 4;
                VIRT_GICD.ipriority[base + 0] = (val & 0xFF) as u8;
                VIRT_GICD.ipriority[base + 1] = ((val >> 8) & 0xFF) as u8;
                VIRT_GICD.ipriority[base + 2] = ((val >> 16) & 0xFF) as u8;
                VIRT_GICD.ipriority[base + 3] = ((val >> 24) & 0xFF) as u8;
            }
            _ => {
                crate::uart_println!("[GICD] MMIO offset=0x{:x}, write={:01x}, val=0x{:x}", offset, is_write, val);
            }
        }
    }
}

pub fn handle_gicc_mmio(ipa: u64, is_write: bool, val: u32, _rt: u8) {
    let offset = (ipa - GUEST_GICC_IPA) as u32;

    unsafe {
        match (offset, is_write) {
            // GICC_CTLR
            (0x000, true) => {
                VIRT_GICC.ctlr = val;
            }
            // GICC_PMR
            (0x004, true) => {
                VIRT_GICC.pmr = val;
            }
            // GICC_IAR (read normalement, ici on log si write)
            (0x00C, true) => {
                VIRT_GICC.iar = val;
            }
            // GICC_EOIR
            (0x010, true) => {
                VIRT_GICC.eoir = val;
            }
            _ => {
                crate::uart_println!("[GICC] MMIO offset=0x{:x}, write={:01x}, val=0x{:x}", offset, is_write, val);
            }
        }
    }
}

pub fn handle_virtio_blk_mmio(ipa: u64, is_write: bool, val: u32, _rt: u8) {
    let offset = (ipa - GUEST_VIRTIO_BLK_IPA) as u32;

    unsafe {
        match (offset, is_write) {
            // MagicValue / Version / DeviceID / VendorID : souvent read-only
            (0x000, false) => { /* MagicValue = 0x74726976 ('virt') */ }
            (0x004, false) => { /* Version = 2 */ }
            (0x008, false) => { /* DeviceID = 2 (blk) */ }
            (0x00C, false) => { /* VendorID = 0 */ }

            // DeviceFeatures
            (0x010, false) => { /* return features */ }
            (0x020, true) => { VIRTIO_BLK.driver_features = val; }

            // QueueSel / QueueNum / QueueReady
            (0x030, true) => { VIRTIO_BLK.queue_sel = val; }
            (0x038, true) => { VIRTIO_BLK.queue_num = val; }
            (0x044, true) => { VIRTIO_BLK.queue_notify = val; }
            (0x050, false) => { /* InterruptStatus read */ }
            (0x064, true) => { VIRTIO_BLK.status = val; }

            // QueueDesc / Avail / Used
            (0x070, true) => { VIRTIO_BLK.queue_desc_low  = val; }
            (0x074, true) => { VIRTIO_BLK.queue_desc_high = val; }
            (0x080, true) => { VIRTIO_BLK.queue_avail_low  = val; }
            (0x084, true) => { VIRTIO_BLK.queue_avail_high = val; }
            (0x090, true) => { VIRTIO_BLK.queue_used_low  = val; }
            (0x094, true) => { VIRTIO_BLK.queue_used_high = val; }

            _ => {
                crate::uart_println!("[virtio-blk] MMIO offset=0x{:x}, write={:01x}, val=0x{:x}", offset, is_write, val);
            }
        }
    }
}
*/

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


//#[no_mangle]
//pub extern "C" fn guest_el1_exception_handler(esr: u64, far: u64, elr: u64) {
//    crate::uart_println!("[EL1] ESR=0x{:016x} FAR=0x{:016x} ELR=0x{:016x}", esr, far, elr);
//}
