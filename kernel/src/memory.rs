use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;
use core::ptr::addr_of_mut;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// initiate heap allocator
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, KERNEL_HEAP_SIZE);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    #[test_case]
    fn test_heap() {
        let mut vec = Vec::new();
        assert_eq!(vec.len(), 0);
        vec.push(1);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], 1);
    }
}
