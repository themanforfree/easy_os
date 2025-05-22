mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_space;
mod page_table;

pub use address::*;
use memory_space::KERNEL_SPACE;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.borrow_mut().activate();
}
