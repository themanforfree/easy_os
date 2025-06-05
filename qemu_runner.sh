#!/bin/bash
GDB=${GDB:-0}
if [ "$CARGO_MANIFEST_DIR" = "" ]; then
  BASE_DIR="$(pwd)"
else
  BASE_DIR="$(dirname "$CARGO_MANIFEST_DIR")"
fi
FS_IMG="${BASE_DIR}/kernel/fs.img"
RUSTSBI="${BASE_DIR}/bootloader/rustsbi.bin"
CMD="
qemu-system-riscv64\
 -nographic\
 -machine virt\
 -bios $RUSTSBI\
 -drive file=$FS_IMG,if=none,format=raw,id=x0\
 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
 -kernel $*
"
if [ "$GDB" -eq 1 ]; then
  CMD+=" -s -S"
fi
echo "$CMD"
exec $CMD
