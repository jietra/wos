pub mod mmio;
pub mod cpu;

use crate::drivers::uart::puts;
use cpu::exceptions::init_exceptions;

pub fn init_arch() {
    puts("| BOOT  | Booting WOS-RISC-V...\n");

    // | CHECK | Execute trap.S
    //crate::debug::cpu::test_trap_entry_direct();

    // --- Initializing trap handling --------------------------------
    puts("| INIT. | Initializing trap handling...\n");
    init_exceptions();

    // | CHECK | Trigger an exception
    unsafe{ crate::debug::cpu::trigger_fault(); }

    puts("\n==========================================================\n");

    puts("\nWOS-RISC-V Firmware v0.1\n");
    puts("(c) 2026 Ulrich Tan\n\n");

    puts("[ OK ] CPU initialized\n");
    puts("[ OK ] Trap handler installed\n");
    puts("[ OK ] UART ready\n\n");

    puts("Booting kernel...\n\n");

    puts("██╗    ██╗ ██████╗  ██████╗\n");
    puts("██║    ██║██╔═══██╗██╔════╝\n");
    puts("██║ █╗ ██║██║   ██║ █████╗ \n");
    puts("██║███╗██║██║   ██║     ██║\n");
    puts("╚███╔███╔╝╚██████╔╝██████╔╝\n");
    puts(" ╚══╝╚══╝  ╚═════╝ ╚═════╝ \n\n");
    puts("  W O S   –   R I S C‑V\n\n");
    
    // --- Welcome message --------------------------------
    puts("\n--------------------------------------\n");
    puts(  "|       Hello from WOS-RISC-V!       |"  );
    puts("\n--------------------------------------\n");

}
