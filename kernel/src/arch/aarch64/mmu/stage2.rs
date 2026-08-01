// src/arch/aarch64/mmu/stage2.rs

use crate::memory::virt::{l2_index, l3_index};
use crate::arch::boot::linker_symbols as ls;
use crate::arch::vm_if::VmArch;

// attrs: normal mem, WB, inner-shareable
const S2_MEMATTR:        u64 = 0;           // use Attr0 from MAIR_EL2
const S2_MEMATTR_DEVICE: u64 = 1;           // Attr1
const S2_SH_INNER:       u64 = 0;           // non-shareable as of now
const S2_AF:             u64 = 1 << 10;

#[repr(C, align(4096))]
pub struct S2Table {
    pub entries: [u64; 512],
}
pub struct AArch64Vm;

impl VmArch for AArch64Vm {
    fn map_page(ipa: u64, pa: u64) {
        unsafe { map_ipa_page(ipa, pa); }
    }
    fn map_device(ipa: u64, pa: u64) {
        unsafe { map_ipa_page_device(ipa, pa); }
    }
    fn get_s2_root() -> u64 {
        unsafe { &S2_ROOT as *const _ as u64 }
    }
    fn get_s2_root_entries() -> *const u64 {
        unsafe { S2_ROOT.entries.as_ptr() }
    }
}

static mut NEXT_TABLE:  u64 = 0;
static mut TABLE_LIMIT: u64 = 0;
#[link_section = ".stage2_root"]
pub static mut S2_ROOT: S2Table = S2Table { entries: [0; 512] };

// ------------------------
// Init MMU stage-2
// ------------------------

/// Initialize VTCR_EL2
pub unsafe fn init_vtcr() {
    crate::uart_println!("Initializing VTCR_EL2...");
    
    let vtcr: u64 =
        (0b010 << 16) |  // PS   = 40-bit PA
        (0b00  << 14) |  // TG0  = 4KB
        (0b11  << 12) |  // SH0  = Inner Shareable
        (0b01  << 10) |  // ORGN0= WB
        (0b01  << 8 ) |  // IRGN0= WB
        //(0b01  << 6 ) |  // SL0  = 0 -> S2_ROOT = L0
        (0b01  << 6 ) |  // SL0 = 1 -> 3 niveaux : L1 -> L2 -> L3
        //(0b10  << 6 ) |  // SL0 = 2 -> 3 niveaux
        //(24);            // T0SZ = 24 -> IPA 40 bits
        (25);            // T0SZ = 25 -> IPA 39 bits
    
    crate::uart_println!("\tvtcr_EL2  = 0x{:016x}", vtcr);

    core::arch::asm!(
        "msr VTCR_EL2, {0}",
        "isb",
        in(reg) vtcr,
    );

    // Check vtcr
    let mut r: u64;
    core::arch::asm!("mrs {0}, VTCR_EL2", out(reg) r);
    crate::uart_println!("\tVTCR_EL2  = 0x{:016x}", r);
}

/// Initialize MAIR_EL2
pub unsafe fn init_mair() {
    crate::uart_println!("Initializing MAIR_EL2...");        
    
    let mair_el2: u64 =
        (0xFF << 0)  | // Attr0 = 0xFF (normal WB)
        (0x04 << 8);   // Attr1 = 0x04 (Device-nGnRE)

    crate::uart_println!("\tmair_EL2  = 0x{:016x}", mair_el2);

    core::arch::asm!(
        "msr mair_el2, {0}",
        "isb",
        in(reg) mair_el2,
    );

    // Check mair
    let mut r: u64;
    core::arch::asm!("mrs {0}, MAIR_EL2", out(reg) r);
    crate::uart_println!("\tMAIR_EL2  = 0x{:016x}", r);
}

/// Initialize VTTBR_EL2
pub unsafe fn init_vttbr() {
    crate::uart_println!("Initializing VTTBR_EL2 with S2_ROOT...");
    
    let root: u64 = &S2_ROOT as *const _ as u64;

    crate::uart_println!("\tS2_ROOT   = 0x{:016x}", root);

    core::arch::asm!(
        "msr vttbr_el2, {}",
        in(reg) root
    );
    
    // Check vttbr
    let mut r: u64;
    core::arch::asm!("mrs {0}, VTTBR_EL2", out(reg) r);
    crate::uart_println!("\tVTTBR_EL2 = 0x{:016x}", r);
}

/// Initialize Stage 2 table pool
pub unsafe fn init_s2_table_pool() {
    crate::uart_println!("Initializing S2 table pool...");

    NEXT_TABLE  = &ls::_stage2_start as *const _ as u64;
    TABLE_LIMIT = &ls::_stage2_end   as *const _ as u64;

    // align NEXT_TABLE on 4K
    NEXT_TABLE = (NEXT_TABLE + 0xFFF) & !0xFFF;

    crate::uart_println!("\tStage‑2 pool next table:  0x{:x}", NEXT_TABLE);
    crate::uart_println!("\tStage‑2 pool table limit: 0x{:x}", TABLE_LIMIT);
    crate::uart_println!("\tStage-2 pool size:        0x{:x}", TABLE_LIMIT-NEXT_TABLE);
}

/// Initializing HCR_EL2
pub unsafe fn init_hcr() {
    crate::uart_println!("Activate virt (HCR_EL2)...");

    // read current hcr
    let mut hcr: u64;
    core::arch::asm!("mrs {}, hcr_el2", out(reg) hcr);
    crate::uart_println!("\tcurrent HCR_EL2 ={}", hcr);
    
    // load new hcr
    let hcr_new: u64 =
        (1 << 31) | // VM  : enable Stage-2
        (1 << 0)  | // RW  : EL1 can be AArch64
        (1 << 10) | // PTW : allow guest page table walks
        (1 << 9)  | // FMO : allow faults
        (1 << 8)  | // IMO : allow interrupts
        (1 << 7);   // AMO : allow data accesses

    core::arch::asm!("msr hcr_el2, {}", in(reg) hcr_new);
    crate::uart_println!("\tnew HCR_EL2     ={}", hcr_new);
}

// ------------------------
// Mapping functions
// ------------------------

pub unsafe fn map_ipa_page(ipa: u64, pa: u64) {
    let i1 = ((ipa >> 30) & 0x1FF) as usize; // L1
    let i2 = ((ipa >> 21) & 0x1FF) as usize; // L2
    let i3 = ((ipa >> 12) & 0x1FF) as usize; // L3

    // L1 -> L2
    let l1_entry = &mut S2_ROOT.entries[i1];
    let l2 = if *l1_entry == 0 {
        let t = alloc_s2_table();
        //*l1_entry = (t as u64) | 0b11; // table descriptor
        *l1_entry = (t as u64 & !0xFFF) | 0b11; // table descriptor
        t
    } else {
        (*l1_entry & !0xFFF) as *mut S2Table
    };

    // L2 -> L3
    let l2_entry = &mut (*l2).entries[i2];
    let l3 = if *l2_entry == 0 {
        let t = alloc_s2_table();
        //*l2_entry = (t as u64) | 0b11; // table descriptor
        *l2_entry = (t as u64 & !0xFFF) | 0b11; // table descriptor
        t
    } else {
        (*l2_entry & !0xFFF) as *mut S2Table
    };

    // L3 -> page (Stage-2)
    let mut desc = pa & !0xFFF;
    desc |= 0b11;               // page descriptor
    desc |= S2_AF;              // Access Flag
    desc |= S2_SH_INNER << 8;   // shareability
    desc |= S2_MEMATTR << 2;    // mem attributes
    desc |= 0b11 << 6;          // S2AP = RW

    (*l3).entries[i3] = desc;

    // Check (debug)
    if ipa == 0x4000_0000 {
        crate::uart_println!(
            "\t\ti2 for IPA 0x40000000 = 0x{:016x}",
            i2
        );
        crate::uart_println!(
            "\t\tS2 L2[i2] for IPA 0x40000000 = 0x{:016x}",
            desc
        );
    }
}

pub unsafe fn map_ipa_page_device(ipa: u64, pa: u64) {
    let i1 = ((ipa >> 30) & 0x1FF) as usize; // L1
    let i2 = ((ipa >> 21) & 0x1FF) as usize; // L2
    let i3 = ((ipa >> 12) & 0x1FF) as usize; // L3

    // L1 -> L2
    let l1_entry = &mut S2_ROOT.entries[i1];
    let l2 = if *l1_entry == 0 {
        let t = alloc_s2_table();
        //*l1_entry = (t as u64) | 0b11; // table descriptor
        *l1_entry = (t as u64 & !0xFFF) | 0b11; // table descriptor
        t
    } else {
        (*l1_entry & !0xFFF) as *mut S2Table
    };

    // L2 -> L3
    let l2_entry = &mut (*l2).entries[i2];
    let l3 = if *l2_entry == 0 {
        let t = alloc_s2_table();
        //*l2_entry = (t as u64) | 0b11; // table descriptor
        *l2_entry = (t as u64 & !0xFFF) | 0b11; // table descriptor
        t
    } else {
        (*l2_entry & !0xFFF) as *mut S2Table
    };

    // L3 -> page (Stage-2)
    let mut desc = pa & !0xFFF;
    desc |= 0b11;                   // page descriptor
    desc |= S2_AF;                  // Access Flag
    desc |= S2_SH_INNER << 8;       // shareability
    desc |= S2_MEMATTR_DEVICE << 2; // <- mem attributes
    desc |= 0b11 << 6;              // S2AP = RW

    (*l3).entries[i3] = desc;

    // Check (debug)
    if ipa == 0x4000_0000 {
        crate::uart_println!(
            "\t\ti2 for IPA 0x40000000 = 0x{:016x}",
            i2
        );
        crate::uart_println!(
            "\t\tS2 L2[i2] for IPA 0x40000000 = 0x{:016x}",
            desc
        );
    }
}

// [Helper] Allocate a table
unsafe fn alloc_s2_table() -> *mut S2Table {
    //let size = core::mem::size_of::<S2Table>() as u64;
    let size = 4096u64; // 4K table

    if NEXT_TABLE + size > TABLE_LIMIT {
        panic!("Stage-2 table pool exhausted");
    }

    let ptr = NEXT_TABLE;
    NEXT_TABLE += size;

    // Check alignment
    debug_assert_eq!(ptr & 0xFFF, 0);

    // Clear table
    let t = ptr as *mut S2Table;
    (*t).entries = [0; 512];

    t
}