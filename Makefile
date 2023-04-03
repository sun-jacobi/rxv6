K=kernel


GDB=riscv64-unknown-elf-gdb
OBJDUMP=riscv64-unknown-elf-objdump
ASM=./src/asm/*.S 
KERNEL_SRC = $(wildcard src/**/*.rs)

KERNEL=target/riscv64gc-unknown-none-elf/debug/kernel
DRIVE=fs.img
LINKER_SCRIPT=src/ld/kernel.ld

# kernel build
$(KERNEL): $(wildcard src/**/*.rs) $(ASM) $(LINKER_SCRIPT)
	cargo build

# create disk image
$(DRIVE): 
	dd if=/dev/zero of=fs.img bs=1m count=32


#### 
# configure QEMU
CPUS=1 # develop in single core for now
QEMU=qemu-system-riscv64
QEMUOPTS = -machine virt -bios none -kernel $(KERNEL) -m 128M -smp $(CPUS) -nographic
QEMUOPTS += -global virtio-mmio.force-legacy=false
QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
QEMUOPTS += -monitor telnet::45454,server,nowait -serial mon:stdio

qemu: $(KERNEL) $(DRIVE)
	$(QEMU) $(QEMUOPTS)

debug: $(KERNEL) $(DRIVE)
	$(QEMU) $(QEMUOPTS) -s -S

gdb:
	$(GDB) -x debug.gdb

dump: 
	$(OBJDUMP) -d $(KERNEL)

quit: 
	telnet localhost 45454
	
##### 
.PHONY: clean
clean:
	cargo clean
	rm fs.img






