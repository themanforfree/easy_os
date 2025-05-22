use crate::{config::MEMORY_END, memory::address::PhysAddr};
use alloc::vec::Vec;

use crate::sync::UPSafeCell;

use super::address::PhysPageNum;

type FrameAllocatorImpl = StackFrameAllocator;
pub static FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
    unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };

#[derive(Debug)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.borrow_mut().dealloc(self.ppn);
    }
}

pub trait FrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameTracker>;
}

pub struct StackFrameAllocator {
    current: PhysPageNum,
    end: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        Self {
            current: PhysPageNum::zero(),
            end: PhysPageNum::zero(),
            recycled: Vec::new(),
        }
    }

    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l;
        self.end = r;
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn)
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#?} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn frame_alloc(&mut self) -> Option<FrameTracker> {
        self.alloc().map(|ppn| FrameTracker { ppn })
    }
}

pub fn init_frame_allocator() {
    unsafe extern "C" {
        safe fn ekernel();
    }
    FRAME_ALLOCATOR.borrow_mut().init(
        // 使用 ekernel 之后的一个 Page 作为可用 FRAME 的起始地址，避免覆盖内核代码
        PhysAddr::new(ekernel as usize).next_page_number(),
        PhysAddr::new(MEMORY_END).page_number(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    pub fn frame_allocator_test() {
        let mut v: Vec<FrameTracker> = Vec::new();
        for i in 0..5 {
            let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
            // println!("{:?}", frame);
            v.push(frame);
        }
        v.clear();
        for i in 0..5 {
            let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
            // println!("{:?}", frame);
            v.push(frame);
        }
        drop(v);
    }
}
