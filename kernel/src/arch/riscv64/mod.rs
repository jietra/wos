pub mod mmio;

use crate::drivers::uart::puts;

pub fn init_arch() {
    puts("[RISC-V] init_arch()\n");
    
    // --- Welcome message --------------------------------
    puts("\n--------------------------------------\n");
    puts(  "|       Hello from WOS-RISC-V!       |"  );
    puts("\n--------------------------------------\n");
}