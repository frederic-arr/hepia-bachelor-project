# Early Boot Environment

One needs two thing to boot:
1) a kernel (`bzImage`)
2) An *initial* root file system that will be mounted at `/`

This initial root file system can take two forms:
- An `initramfs` loaded by the bootloader along the kernel at a specific offset in memory, or;
- A disk of some sort as `/dev/sda`, `/dev/vda2`, etc.

If it is an `initramfs`, it will be loaded entierly into RAM, and its goal should be to embark the stricti minimum to find and mount the real root filesystem (e.g. needing to load kernel modules, or using some fs that don't directly exist as a physical device such as NFS).



Once the kernel is done, it calls `/init`. At this point, nothing exists: no `/dev`*, no `/proc`, etc.
