K=kernel



TARGET=riscv64gc-unknown-none-elf



####
# configure GCC
TOOLPREFIX=riscv64-unknown-elf-
CC=$(TOOLPREFIX)gcc
CFLAGS = -Wall -Werror -O -fno-omit-frame-pointer -ggdb -gdwarf-2
CFLAGS += -MD
CFLAGS += -mcmodel=medany
CFLAGS += -ffreestanding -fno-common -nostdlib -mno-relax
CFLAGS += -I. -fno-stack-protector -fno-pie -no-pie





# Kernel build
ASM=./src/asm/*.S 

KERNEL_LIB=-lrxv6 -lgcc
KERNEL_LIBS=./target/riscv64gc-unknown-none-elf/debug/
KERNEL_LIB_OUT=./target/riscv64gc-unknown-none-elf/debug/libxv6.a
KERNEL=kernel.elf
LINKER_SCRIPT=./src/ld/kernel.ld


$(KERNEL_LIB_OUT):
	cargo build


$(KERNEL) : $(KERNEL_LIB_OUT) $(ASM)
	$(CC) $(CFLAGS) -T$(LINKER_SCRIPT) -o $(KERNEL) $(ASM) $(KERNEL_LIB) -L$(KERNEL_LIBS)


#### 
# configure QEMU
CPUS=4
QEMU=qemu-system-riscv64
QEMUOPTS = -machine virt -bios none -kernel $(KERNEL) -m 128M -smp $(CPUS) -nographic
QEMUOPTS += -global virtio-mmio.force-legacy=false
QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

qemu: $(KERNEL) 
	dd if=/dev/zero of=fs.img bs=1m count=32
	$(QEMU) $(QEMUOPTS)
	
##### 
.PHONY: clean
clean:
	cargo clean
	rm *.d *.elf






