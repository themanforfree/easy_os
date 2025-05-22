pub const KERNEL_HEAP_SIZE: usize = 0x20_0000; // 2 MiB
pub const MEMORY_END: usize = 0x8800_0000; // TODO: get this from device tree
pub const PAGE_SIZE: usize = 0x1000; // 4 KiB

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
