fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if arch == "riscv64" {
        cc::Build::new()
            .file("src/arch/riscv64/boot/start.S")
            .file("src/arch/riscv64/cpu/trap.S") 
            .compiler("clang")
            .flag("--target=riscv64-unknown-elf")
            .flag("-march=rv64gc")
            .flag("-mabi=lp64d")
            .compile("riscv64_objs");
    }

    if arch == "aarch64" {
        cc::Build::new()
            .file("src/arch/aarch64/boot/start.S")
            .file("src/arch/aarch64/cpu/exception_vectors.S")
            .compiler("clang")
            .flag("--target=aarch64-unknown-elf")
            .compile("aarch64_objs");
    }
}
