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

#[macro_export]
macro_rules! uart_println {
    ($s:expr) => {
        $crate::utils::print::println($s);
    };
    ($s:expr, $($arg:expr),+) => {{
        $crate::utils::print::print($s);
        $(
            $crate::utils::print::put_hex($arg as u64);
            $crate::utils::print::print(" ");
        )+
        $crate::utils::print::print("\n");
    }};
}

/* For formatted println: needs to be adjusted
pub fn print_fmt(fmt: &str, args: &[u64]) {
    let mut arg_i = 0;

    let bytes = fmt.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'{' && i + 1 < bytes.len() && bytes[i+1] == b'}' {
            // print next argument in hex
            if arg_i < args.len() {
                put_hex(args[arg_i]);
                arg_i += 1;
            } else {
                print("<?>");
            }
            i += 2;
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
*/