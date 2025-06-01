set confirm off
set architecture riscv:rv64
symbol-file target/riscv64gc-unknown-none-elf/debug/kernel
add-symbol-file target/riscv64gc-unknown-none-elf/debug/init
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes
