use alloc::{string::String, vec, vec::Vec};
use bitflags::bitflags;

use crate::{
    config::PAGE_SIZE,
    memory::{
        PhysAddr,
        frame_allocator::{FRAME_ALLOCATOR, FrameAllocator},
    },
};

use super::{PhysPageNum, VirtAddr, VirtPageNum, frame_allocator::FrameTracker};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0; // Valid
        const R = 1 << 1; // Readable
        const W = 1 << 2; // Writable
        const X = 1 << 3; // Executable
        const U = 1 << 4; // User-accessible
        const G = 1 << 5; // Global
        const A = 1 << 6; // Accessed
        const D = 1 << 7; // Dirty
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: (usize::from(ppn) << 10) | flags.bits() as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum::new(self.bits >> 10 & ((1usize << 44) - 1))
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits as u8)
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn is_readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn is_writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn is_executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    fn is_readonly(&self) -> bool {
        self.frames.is_empty()
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;
        for (i, &idx) in indexes.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[idx];
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        None
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        debug_assert!(
            !self.is_readonly(),
            "Page table is readonly, cannot create new page table entry"
        );
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;
        for (i, &idx) in indexes.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[idx];
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        None
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {vpn:?} is already mapped");
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    #[allow(dead_code)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {vpn:?} is not mapped");
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).copied()
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.translate(va.page_number())
            .map(|pte| PhysAddr::from(pte.ppn()) + va.page_offset())
    }

    // pub fn translate_ptr<T>(&self, ptr: *const T) -> *const T {
    //     self.translate_va(VirtAddr::new(ptr as usize))
    //         .map(|pa| pa.as_ptr())
    //         .unwrap()
    // }

    pub fn translate_mut_ptr<T>(&self, ptr: *mut T) -> *mut T {
        self.translate_va(VirtAddr::new(ptr as usize))
            .map(|pa| pa.as_mut_ptr())
            .unwrap()
    }

    pub fn read_c_str(&self, ptr: *const u8) -> Option<String> {
        let mut s = String::new();
        let mut va = VirtAddr::new(ptr as usize);
        loop {
            let ch = *self.translate_va(va)?.get_mut::<u8>(); // TODO: optimize this
            if ch == 0 {
                break;
            }
            s.push(ch as char); // TODO: optimize this to support UTF-8
            va += 1;
        }
        Some(s)
    }

    pub fn token(&self) -> usize {
        8usize << 60 | usize::from(self.root_ppn)
    }

    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::new(token & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    pub fn copy_out(&self, ptr: VirtAddr, buf: &[u8]) {
        let mut start_va = ptr;
        let end_va = start_va + buf.len();
        let mut written = 0;

        while start_va < end_va {
            let vpn = start_va.page_number();
            let page_offset = start_va.page_offset();
            let dst = self.translate(vpn).unwrap().ppn().get_bytes_array();

            let bytes_to_copy = usize::min(PAGE_SIZE - page_offset, end_va - start_va);
            let src_slice = &buf[written..written + bytes_to_copy];

            dst[page_offset..page_offset + bytes_to_copy].copy_from_slice(src_slice);

            start_va += bytes_to_copy;
            written += bytes_to_copy;
        }
    }

    pub fn translate_bytes_buffer(&self, ptr: VirtAddr, len: usize) -> UserBuffer {
        let mut buffer = Vec::with_capacity(len);

        let mut start_va = ptr;
        let max_end_va = start_va + len;
        while start_va < max_end_va {
            let vpn = start_va.page_number();
            let page_offset = start_va.page_offset();
            let src = self.translate(vpn).unwrap().ppn().get_bytes_array();

            let bytes_to_copy = usize::min(PAGE_SIZE - page_offset, max_end_va - start_va);
            buffer.push(&mut src[page_offset..page_offset + bytes_to_copy]);

            start_va += bytes_to_copy;
        }

        UserBuffer { buffer }
    }
}

// TODO: use a more flexible abstraction for user buffers
pub struct UserBuffer {
    pub buffer: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn len(&self) -> usize {
        self.buffer.iter().map(|b| b.len()).sum()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut &'static mut [u8]> {
        self.buffer.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &&'static mut [u8]> {
        self.buffer.iter()
    }
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffer,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}

pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}
