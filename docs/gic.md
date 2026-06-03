# Interrupt Controller (GIC)

## GIC Version
This kernel uses **GICv2**, *not GICv3*. Indeed, the default QEMU `virt` machine exposes a GIC compatible with:
```bash
compatible = "arm,cortex-a15-gic";
```

This corresponds to **GICv2/GIC-400**, the interrupt controller used in Cortex‑A15‑class systems.

No GICv3 redistributors (`GICR`) or ITS nodes are present in the Device Tree.

## Memory Mapping (from the DTB)
The DTB provides the following `reg` entries for the interrupt controller:
```bash
reg = <0x00000000 0x08000000   // GICD base
       0x00000000 0x00010000   // GICD size
       0x00000000 0x08010000   // GICC base
       0x00000000 0x00010000>; // GICC size
```

Therefore:
- GICD_BASE = 0x08000000
- GICC_BASE = 0x08010000

These addresses are used by the kernel to initialize the distributor and CPU interface.

## Important Note About QEMU
If you run QEMU with:
```bash
-machine virt
```
You get **GICv2** (default).

If you run QEMU with:
```bash
-machine virt,gic-version=3
```
You get **GICv3**, which this kernel does not support.

Attempting to boot with GICv3 will result in crashes (invalid register accesses, data aborts, etc.).
