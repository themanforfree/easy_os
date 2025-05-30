mod address;
mod frame_allocator;
mod heap_allocator;
mod map_area;
mod memory_space;
mod page_table;

pub use self::address::*;
pub use self::map_area::MapPermission;
pub use self::memory_space::{KERNEL_SPACE, MemorySpace};
pub use self::page_table::PageTable;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.borrow_mut().activate();
}
