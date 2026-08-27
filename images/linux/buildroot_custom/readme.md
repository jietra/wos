# Build a custom minimal guest Linux for hWALT

Use buildroot:
```bash
git clone https://github.com/buildroot/buildroot.git
```

## Linux.config
Create a custom linux.config file for your system. You can use the example file provided here in `board/xWALT/linux.config`. Copy it into the `buildroot` directory:

```bash
mkdir -p board/xWALT
cp <path_to_buildroot_custom>/board/xWALT/linux.config board/xWALT
```

## defconfig file
You can simply 'hack' the `configs/qemu_aarch64_virt_defconfig` file provided by buildroot with the one provided here in `configs/`, or create your own defconfig.

Then build your linux binary:
```bash
make qemu_aarch64_virt_defconfig
make -j$(nproc)
```

Copy `output/Image`, `output/rootfs.cpio` (and all other files needed, depending on your xWALT `config.rs`) into the xWALT `images/linux` directory.

## IMPORTANT: adapt your guest `.dtb`
Adapt the xWALT `images/linux/guest1.dts` file to the size of your linux binary: adapt the line `linux,initrd-end   = <0x0 0x4358f3ff>;`. You can use hWALT initial logs to find the correct `initrd-end` address.

And do not forget to rebuild your `.dtb`:
```bash
dtc -I dts -O dtb guest1.dts -o guest1.dtb
```