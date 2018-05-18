TARGET=target/kernel.bin
SRCS=$(shell find src -name "*.rs") spec.json

#RUN_CMD=qemu-system-x86_64 -nographic -curses -drive format=raw,file=$(TARGET)
RUN_CMD=qemu-system-x86_64 -d cpu_reset -drive format=raw,file=$(TARGET)

.PHONY: build
build: $(TARGET)

.PHONY: run
run: build
	$(RUN_CMD)

.PHONY: debug
debug: build
	$(RUN_CMD) -s -S

$(TARGET): $(SRCS)
	bootimage build
