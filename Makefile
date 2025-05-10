TARGET := riscv64gc-unknown-none-elf
MODE := debug
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
GDB_BIN := gdb

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

kernel:
	@cargo build $(MODE_ARG)

run: kernel
	@./qemu_runner.sh $(KERNEL_ELF)

gdbserver: kernel
	@GDB=1 ./qemu_runner.sh $(KERNEL_ELF)

gdbclient:
	@$(GDB_BIN) -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

.PHONY: kernel run gdbserver gdbclient
