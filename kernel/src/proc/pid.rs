use alloc::vec::Vec;

use crate::sync::UPSafeCell;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> =
        unsafe { UPSafeCell::new(PidAllocator::new()) };
}

pub struct PidAllocator {
    next_pid: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    pub fn new() -> Self {
        Self {
            next_pid: 1, // Start from 1, 0 is reserved for the kernel
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> PidTracker {
        if let Some(pid) = self.recycled.pop() {
            PidTracker(pid)
        } else {
            let pid = self.next_pid;
            self.next_pid += 1;
            PidTracker(pid)
        }
    }

    pub fn free(&mut self, pid: usize) {
        if pid > 0 && !self.recycled.contains(&pid) {
            self.recycled.push(pid);
        }
    }
}

pub struct PidTracker(pub usize);

impl Drop for PidTracker {
    fn drop(&mut self) {
        PID_ALLOCATOR.borrow_mut().free(self.0);
    }
}
