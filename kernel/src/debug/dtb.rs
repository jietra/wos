// src/dtb.rs
// DTB parser, for debugging purposes, in order to build GIC

use crate::drivers::uart::{puts, putc};
use crate::utils::print::put_hex_ln;

/*
// In case of missalignment. In this case, use dtb = DTB.0 to get the aligned slice,
// but since we use include_bytes! which places the data in the .rodata section,
// it should already be properly aligned and we can directly use the included bytes without copying.
// If we were to copy it, we would need to ensure that the destination buffer is properly aligned
// (e.g. using a static mutable buffer with #[repr(align(4))] or similar) and then copy the data from the included bytes to that buffer at runtime before parsing it.
#[repr(align(4))]
pub struct Aligned<T>(pub T);
// DTB aligned, no copying needed since we can directly use the included bytes (which are already in the .rodata section and thus accessible at runtime)
pub static DTB: Aligned<&[u8]> = Aligned(include_bytes!("../virt.dtb"));
*/
pub static DTB: &[u8] = include_bytes!("../../virt.dtb");

// --- Debug: print some info about the DTB (for debugging purposes) --------------------------------
pub unsafe fn debug_dtb() {
    let dtb = DTB;
    let addr = dtb.as_ptr() as u64;
    puts("\tDTB addr \t= 0x"); put_hex_ln(addr);

    // First 4 bytes of the DTB should be the magic number 0xD00DFEED in big-endian
    let magic = u32::from_be_bytes([dtb[0], dtb[1], dtb[2], dtb[3]]);
    puts("\tDTB magic \t= 0x"); put_hex_ln(magic as u64);
}

// --- Helper functions for DTB parsing (for debugging purposes) --------------------------------
const FDT_BEGIN_NODE: u32 = 1;
const FDT_END_NODE: u32 = 2;
const FDT_PROP: u32 = 3;
const FDT_NOP: u32 = 4;
const FDT_END: u32 = 9;

#[repr(C)]
#[derive(Debug)]
struct FdtHeader {
    //magic: u32,
    //totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    //off_mem_rsvmap: u32,
    //version: u32,
    //last_comp_version: u32,
    //boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

fn print_bytes_as_text(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        putc(bytes[i]);
        i += 1;
    }
}

fn read_header(dtb: &[u8]) -> FdtHeader {
    FdtHeader {
        //magic: u32::from_be_bytes(dtb[0..4].try_into().unwrap()),
        //totalsize: u32::from_be_bytes(dtb[4..8].try_into().unwrap()),
        off_dt_struct: u32::from_be_bytes(dtb[8..12].try_into().unwrap()),
        off_dt_strings: u32::from_be_bytes(dtb[12..16].try_into().unwrap()),
        //off_mem_rsvmap: u32::from_be_bytes(dtb[16..20].try_into().unwrap()),
        //version: u32::from_be_bytes(dtb[20..24].try_into().unwrap()),
        //last_comp_version: u32::from_be_bytes(dtb[24..28].try_into().unwrap()),
        //boot_cpuid_phys: u32::from_be_bytes(dtb[28..32].try_into().unwrap()),
        size_dt_strings: u32::from_be_bytes(dtb[32..36].try_into().unwrap()),
        size_dt_struct: u32::from_be_bytes(dtb[36..40].try_into().unwrap()),
    }
}

// --- DTB parsing (for debugging purposes) --------------------------------

pub fn parse_dtb() {
    puts("\tParsing DTB...\n");
    let dtb = DTB;
    let h = read_header(dtb);
    let mut p = h.off_dt_struct as usize;
    let end = p + h.size_dt_struct as usize;

    let strings = &dtb[
        h.off_dt_strings as usize ..
        h.off_dt_strings as usize + h.size_dt_strings as usize
    ];

    while p < end {
        let token = u32::from_be_bytes(dtb[p..p+4].try_into().unwrap());
        p += 4;

        match token {
            FDT_BEGIN_NODE => { // FDT_BEGIN_NODE
                let mut i = p;
                // Safety: we never pass the buffer
                while i < dtb.len() && dtb[i] != 0 {
                    i += 1;
                }
                if i == p {
                    // Root node : empty name, we can just print "/"
                    puts("\t\t/\n");
                } else {
                    // We print byte by byte to avoid the need for a temporary buffer and UTF-8 validation (since node names are usually ASCII and we just want to print them as-is)
                    let mut j = p;
                    while j < i {
                        let c = dtb[j] as char;
                        // Optionally, we could also replace non-printable characters with a placeholder (e.g. '?') to avoid printing garbage, but for now we just print them as-is
                        putc(c as u8);
                        j += 1;
                    }
                    puts("\n");
                }
                // alignment 4
                p = (i + 1 + 3) & !3;
            }

            FDT_PROP => { // FDT_PROP
                let len = u32::from_be_bytes(dtb[p..p+4].try_into().unwrap()) as usize;
                let nameoff = u32::from_be_bytes(dtb[p+4..p+8].try_into().unwrap()) as usize;
                p += 8;

                // Safety
                if nameoff >= strings.len() {
                    puts("\t\tERROR: nameoff out of range!\n");
                    puts("\t\t"); put_hex_ln(nameoff as u64);
                    break;
                }
                
                // Retrieve the property name from the strings section using nameoff, we read until the first '\0' to get the name as a byte slice (since we want to compare it as bytes, not necessarily valid UTF-8)
                let mut k = nameoff;
                while k < strings.len() && strings[k] != 0 {
                    k += 1;
                }
                let name_bytes = &strings[nameoff..k];

                // For debugging, we print the property name and value (as hex if it's a reg property, or as a string if it's a compatible property)
                puts("\t\tProp name: ");
                puts("\t\t"); print_bytes_as_text(name_bytes);
                puts("\n");

                // For comparison, we remain byte per byte to avoid the need for temporary buffers and UTF-8 validation. We check if the property name is "compatible" or "reg" (which are the most common properties we're interested in) and print the value accordingly.
                let is_compatible = name_bytes == b"compatible";
                let is_reg        = name_bytes == b"reg";

                puts("\t\t  Prop: ");
                puts("\t\t"); print_bytes_as_text(name_bytes);
                puts("\t\t = ");

                if is_compatible {
                    // value = dtb[p .. p+len], we print until the first '\0'
                    let mut j = 0;
                    while j < len {
                        let c = dtb[p + j];
                        if c == 0 {
                            break;
                        }
                        putc(c);
                        j += 1;
                    }
                }

                if is_reg {
                    puts("\t\t<");
                    let mut i2 = 0;
                    while i2 + 4 <= len {
                        let v = u32::from_be_bytes(dtb[p+i2..p+i2+4].try_into().unwrap());
                        puts("\t\t"); put_hex_ln(v as u64);
                        i2 += 4;
                    }
                    puts("\t\t>");
                }

                puts("\n");

                p = (p + len + 3) & !3;
            }


            FDT_END_NODE => puts("\t\tEnd node\n\t\t--------------------------------------------\n"),
            FDT_END => { puts("\t\tEND\n\t\t============================================\n\n"); break; }
            FDT_NOP => {} // NOP
            _ => { puts("\t\tUnknown token\n"); break; }
        }
    }
}