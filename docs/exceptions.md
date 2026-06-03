# ARMv8 Exception Handling in WOS

WOS implements a complete and clean exception handling pipeline for **AArch64 EL1**, including:
- exception vector table (synchronous, IRQ, FIQ, SError)
- decoding of ESR_EL1 (Exception Syndrome Register)
- reporting of ELR_EL1 (faulting instruction)
- reporting of FAR_EL1 (faulting address)
- handling of:
  - synchronous exceptions
  - data aborts
  - instruction aborts
  - alignment faults
  - FP/SIMD traps
  - unknown exceptions

---

## 1. Exception Vector Table

The vector table is defined in `arch/aarch64/exception_vectors.S`.
It contains the four mandatory entries for EL1:
- Synchronous exceptions  
- IRQ  
- FIQ  
- SError  

Each entry branches to a Rust handler.

---

## 2. Rust Exception Handlers

The Rust handlers live in `arch/aarch64/exceptions.rs`.
They extract and decode:
- **ESR_EL1** — exception class + syndrome  
- **ELR_EL1** — address of the faulting instruction  
- **FAR_EL1** — faulting virtual address (for aborts)

> **Note:** Detailed diagnostic printing, including:
>- exception class (EC)
>- instruction length (IL)
>- fault status code (FSC)
>- faulting VA (for aborts)
>- return address (ELR)
>
> will be expanded in future revisions. This will make debugging easier.

---

## 3. Supported Exception Types

### Synchronous exceptions
- undefined instruction  
- illegal execution state  
- SVC (supervisor call)  
- instruction abort  
- data abort  
- alignment fault  
- FP/SIMD trap  

### IRQ
> **Planned:** IRQ handling will be integrated with the GICv2 driver.

### FIQ
Not used on QEMU `virt`.

### SError
Reported but not recovered.

---

## 4. FP/SIMD Trap Handling

WOS enables FP/SIMD in EL1 and handles traps caused by:
- using FP registers before enabling FP  
- lazy FP context switching (future work)

---

## 5. Data Abort Handling

Data aborts are fully decoded:
- translation faults  
- permission faults  
- alignment faults  
- access flag faults  

> **Note:** The kernel will soon include extended debug output for data aborts (VA, FSC, EC, ELR).

---

## 6. Future Work
 
- IRQ routing to Rust handlers  
- timer interrupts  
- user‑space exception handling  
- SVC‑based syscall interface  