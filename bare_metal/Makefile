.PHONY: all run clean run-nvme

TARGET      := riscv64imac-unknown-none-elf
KERNEL_FILE := target/$(TARGET)/release/os
DEBUG_FILE  ?= $(KERNEL_FILE)

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

all:
	@rm -f kernel-qemu
	cargo build --release
	cp $(KERNEL_FILE) kernel-qemu

run: all
	cp fat32.img nvme.img
	qemu-system-riscv64 \
	-machine virt \
	-bios default \
	-drive file=nvme.img,if=none,id=nvm \
	-device nvme,serial=deadbeef,drive=nvm \
	-drive file=fat32.img,if=none,format=raw,id=x0 \
	-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
	-kernel kernel-qemu \
	-nographic \
	-smp 1 -m 2G
	rm nvme.img

clean:
	rm kernel-qemu
	rm $(KERNEL_FILE)

fs-img: 
	@rm -f fat32.img
	@dd if=/dev/zero of=fat32.img count=100 bs=1M	# 40M
	@mkfs.vfat fat32.img -F 32