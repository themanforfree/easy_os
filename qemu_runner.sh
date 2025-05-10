#!/bin/bash
GDB=${GDB:-0}
if [ "$CARGO_MANIFEST_DIR" = "" ]; then
  BASE_DIR="$(pwd)"
else
  BASE_DIR="$(dirname "$CARGO_MANIFEST_DIR")"
fi

RUSTSBI="${BASE_DIR}/bootloader/rustsbi.bin"
CMD="qemu-system-riscv64 -nographic -machine virt -bios $RUSTSBI -kernel $*"
if [ "$GDB" -eq 1 ]; then
  CMD+=" -s -S"
fi
echo "$CMD"
exec $CMD