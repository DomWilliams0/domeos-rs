TARGET=target/kernel.bin
SRCS=$(shell find src -name "*.rs") spec.json

.PHONY: build
build: $(TARGET)

.PHONY: run
run: build
	qemu-system-x86_64 -drive format=raw,file=$(TARGET)

$(TARGET): $(SRCS)
	bootimage build
