KERNEL          = target/spec/debug/libdomeos.a
OS              = target/domeos.bin
TARGET          = target/domeos.iso
SRCS            = $(shell find src -name "*.rs") spec.json

RUN_CMD         = qemu-system-x86_64 -d cpu_reset -D qemu-logfile -monitor stdio -cdrom $(TARGET)

ifeq ($(CURSES),1)
	RUN_CMD    := $(RUN_CMD) -nographic -curses
endif

CC              = x86_64-elf-gcc
LD              = x86_64-elf-ld

CFLAGS          = -ffreestanding -lgcc
LDFLAGS         = -n -Tlinker.ld -g -nostdlib

NASM_FLAGS      = -felf64 -Wall

BOOTLOADER_SRC  = $(shell find src/boot/*.s)
BOOTLOADER      = $(patsubst %.s, target/%.o, $(notdir $(BOOTLOADER_SRC)))

ISO_DIR         = target/isofiles
GRUB_DIR        = $(ISO_DIR)/boot/grub
GRUB_CFG        = $(GRUB_DIR)/grub.cfg

.PHONY: build
build: $(TARGET)

.PHONY: mk_iso

.PHONY: run
run: build
	$(RUN_CMD)

.PHONY: debug
debug: build
	$(RUN_CMD) -s -S

.PHONY: clean
clean:
	rm -rf $(BOOTLOADER) $(ISO_DIR) $(TARGET) $(KERNEL) $(OS)
	cargo clean -p domeos

$(OS): $(BOOTLOADER) $(KERNEL)
	$(LD) $(LDFLAGS) -o $@ $^

$(KERNEL): $(SRCS)
	RUST_TARGET_PATH=$(PWD) xargo build --target spec

target/%.o: src/boot/%.s
	nasm $(NASM_FLAGS) $< -o $@

$(GRUB_CFG):
	mkdir -p $(GRUB_DIR)
	echo -e "set timeout=0\nset default=0\nmenuentry \"domeos\" {\nmultiboot2 /boot/$(notdir $(OS))\nboot\n}" > $@


$(TARGET): $(OS) $(GRUB_CFG)
	cp $(OS) $(GRUB_DIR)/../
	grub-mkrescue -o $@ $(ISO_DIR)
