pub const KERNEL_HEAP_SIZE: usize = 0x20_0000; // 2 MiB
pub const MEMORY_END: usize = 0x8800_0000; // TODO: get this from device tree
pub const PAGE_SIZE: usize = 0x1000; // 4 KiB
pub const USER_STACK_SIZE: usize = PAGE_SIZE * 2;
pub const KERNEL_STACK_SIZE: usize = PAGE_SIZE * 2;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_FRAME: usize = TRAMPOLINE - PAGE_SIZE;
pub const CLOCK_FREQ: usize = 12500000;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
