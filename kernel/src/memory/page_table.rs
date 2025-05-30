use alloc::{vec, vec::Vec};
use bitflags::bitflags;

use crate::{
    config::PAGE_SIZE,
    memory::frame_allocator::{FRAME_ALLOCATOR, FrameAllocator},
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

    pub fn token(&self) -> usize {
        8usize << 60 | usize::from(self.root_ppn)
    }

    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::new(token & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    pub fn copy_out(&self, start_vpn: VirtPageNum, end_vpn: VirtPageNum, data: &[u8]) {
        let mut offset = 0;
        let len = data.len();
        for vpn in start_vpn..end_vpn {
            let src = &data[offset..len.min(offset + PAGE_SIZE)];
            let dst = &mut self.translate(vpn).unwrap().ppn().get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            offset += PAGE_SIZE;
            if offset >= len {
                break;
            }
        }
    }

    // pub fn copy_in(&self, start_va: VirtAddr, len: usize) -> Vec<u8> {
    //     let mut dst = Vec::with_capacity(len);
    //     let mut copied = 0;
    //     for vpn in start_va.page_number().. {
    //         // every page
    //         let ppn = self.translate(vpn).unwrap().ppn();
    //         let src = ppn.get_bytes_array();

    //         let start = if copied == 0 {
    //             start_va.page_offset()
    //         } else {
    //             0
    //         };
    //         let end = if copied + PAGE_SIZE > len {
    //             len - copied + start
    //         } else {
    //             PAGE_SIZE
    //         };
    //         dst.extend_from_slice(&src[..len]);
    //         copied += end - start;
    //         if copied >= len {
    //             break;
    //         }
    //     }
    //     dst
    // }

    pub fn copy_in(&self, ptr: *const u8, len: usize) -> Vec<u8> {
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let start_va = VirtAddr::new(start);
            let mut vpn = start_va.page_number();
            let ppn = self.translate(vpn).unwrap().ppn();
            vpn.step();
            let mut end_va: VirtAddr = vpn.into();
            end_va = end_va.min(VirtAddr::new(end));

            let src = ppn.get_bytes_array();
            if end_va.page_offset() == 0 {
                v.extend_from_slice(&src[start_va.page_offset()..]);
            } else {
                v.extend_from_slice(&src[start_va.page_offset()..end_va.page_offset()]);
            }
            start = end_va.into();
        }
        v
    }
}
