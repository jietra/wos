// src/arch/aarch64/mmu/stage2.rs

use crate::memory::virt::{l2_index, l3_index};
use crate::arch::boot::linker_symbols as ls;
use crate::arch::vm_if::VmArch;

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

static mut NEXT_TABLE: u64 = 0;
static mut TABLE_LIMIT: u64 = 0;

#[repr(C, align(4096))]
pub struct S2Table {
    pub entries: [u64; 512],
}

#[link_section = ".stage2_root"]
pub static mut S2_ROOT: S2Table = S2Table { entries: [0; 512] };

pub unsafe fn init_s2_table_pool() {
    crate::println!("init S2 table pool...");
    NEXT_TABLE  = &ls::_stage2_start as *const _ as u64;
    TABLE_LIMIT = &ls::_stage2_end   as *const _ as u64;

    // align NEXT_TABLE on 4K
    NEXT_TABLE = (NEXT_TABLE + 0xFFF) & !0xFFF;

    crate::uart_println!("\tStage‑2 pool: 0x{:x} → 0x{:x}",
        NEXT_TABLE, TABLE_LIMIT);

    crate::uart_println!("\tS2_ROOT @ 0x{:016x}", &S2_ROOT as *const _ as u64);
}

// attrs: normal mem, WB, inner-shareable
const S2_MEMATTR: u64 = 0;// use Attr0 from MAIR_EL2
const S2_SH_INNER: u64 = 0;// non-shareable as of now
const S2_AF: u64      = 1 << 10;

const S2_MEMATTR_DEVICE: u64 = 1; // Attr1

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

    if ipa == 0x4000_0000 {
        crate::uart_println!(
            "S2 L2[{}] for IPA 0x40000000 = 0x{:016x}",
            i2,
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
    desc |= S2_MEMATTR_DEVICE << 2; // mem attributes
    desc |= 0b11 << 6;              // S2AP = RW

    (*l3).entries[i3] = desc;

    if ipa == 0x4000_0000 {
        crate::uart_println!(
            "S2 L2[{}] for IPA 0x40000000 = 0x{:016x}",
            i2,
            desc
        );
    }
}

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