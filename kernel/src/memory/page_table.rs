use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;

use crate::memory::frame_allocator::{FRAME_ALLOCATOR, FrameAllocator};

use super::{PhysPageNum, VirtPageNum, frame_allocator::FrameTracker};

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
        self.frames.len() == 0
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[indexes[i]];
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
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[indexes[i]];
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
        assert!(!pte.is_valid(), "vpn {:?} is already mapped", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is not mapped", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).copied()
    }

    pub fn token(&self) -> usize {
        8usize << 60 | usize::from(self.root_ppn)
    }
}
