TARGET=target/kernel.bin
SRCS=$(shell find src -name "*.rs") spec.json

.PHONY: run
run: build
	qemu-system-x86_64 -drive format=raw,file=$(TARGET)

.PHONY: build
build: $(TARGET)


$(TARGET): $(SRCS)
	bootimage build
