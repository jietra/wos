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

// For formatted println
pub fn print_fmt(fmt: &str, args: &[u64]) {
    let mut arg_i = 0;
    let bytes = fmt.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Find closing '}'
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'}' {
                j += 1;
            }
            if j >= bytes.len() {
                // malformed, print '{' and continue
                putc(b'{');
                i += 1;
                continue;
            }

            // Extract inside {...}
            let inside = &bytes[i+1 .. j];

            // Case 1: simple {}
            if inside.len() == 0 {
                if arg_i < args.len() {
                    put_hex(args[arg_i]);
                    arg_i += 1;
                } else {
                    print("<?>");
                }
            }
            // Case 2: format like :016x or :x
            else if inside[0] == b':' {
                let fmtstr = core::str::from_utf8(&inside[1..]).unwrap_or("");
                let mut width = 0;
                let mut hex = false;

                // parse width + x
                for c in fmtstr.chars() {
                    if c.is_ascii_digit() {
                        width = width * 10 + (c as u32 - '0' as u32);
                    } else if c == 'x' || c == 'X' {
                        hex = true;
                    }
                }

                if hex {
                    if arg_i < args.len() {
                        let v = args[arg_i];
                        arg_i += 1;

                        // if no width -> default to width = 16
                        if width == 0 {
                            width = 16;
                        }

                        // print padded hex
                        // full 16‑digit hex buffer
                        let mut buf = [0u8; 16];
                        for k in 0..16 {
                            let nibble = ((v >> ((15 - k) * 4)) & 0xF) as u8;
                            buf[k] = match nibble {
                                0..=9 => b'0' + nibble,
                                _ => b'a' + (nibble - 10),
                            };
                        }

                        let start = if width < 16 { 16 - width as usize } else { 0 };
                        for k in start..16 {
                            putc(buf[k]);
                        }
                    } else {
                        print("<?>");
                    }
                }
            }
            else {
                // Unknown format
                print("<?>");
            }

            i = j + 1;
        } else {
            putc(bytes[i]);
            i += 1;
        }
    }

    putc(b'\n');
}

#[macro_export]
macro_rules! uart_println {
    ($fmt:expr $(, $arg:expr)* ) => {{
        let args_slice: &[u64] = &[$($arg as u64),*];
        $crate::utils::print::print_fmt($fmt, args_slice);
    }};
}