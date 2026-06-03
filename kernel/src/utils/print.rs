use crate::drivers::uart::putc;

pub fn put_hex(v: u64) {
    // Print a 64-bit value in hexadecimal (16 hex digits) without using any Rust formatting macros since we're in no_std. In a real kernel, you'd want to implement a proper formatting function to make this easier.
    for i in (0..16).rev() {
        let nibble = ((v >> (i * 4)) & 0xF) as u8;
        let c = match nibble {
            0..=9 => b'0' + nibble,
            10..=15 => b'a' + (nibble - 10),
            _ => b'?', // impossible
        };
        putc(c);
    }
}

pub fn put_hex_ln(v: u64) {
    put_hex(v);
    putc(b'\n');
}

pub fn print(s: &str) {
    for b in s.as_bytes() {
        putc(*b);
    }
}

pub fn println(s: &str) {
    print(s);
    putc(b'\n');
}

#[macro_export]
macro_rules! print {
    ($s:expr) => {
        $crate::utils::print::print($s);
    };
}

#[macro_export]
macro_rules! println {
    ($s:expr) => {
        $crate::utils::print::println($s);
    };
}