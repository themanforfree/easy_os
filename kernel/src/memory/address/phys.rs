//! Physical Address Format
//! ```text
//! ┌────────────────────────────────┬──────────────┐
//! │     Physical Page Number       │ Page Offset  │
//! └────────────────────────────────┴──────────────┘
//!  55                            12 11           0
//! ```

use core::fmt::Debug;

use crate::{config::PAGE_SIZE, memory::page_table::PageTableEntry};

const PA_WIDTH_SV39: usize = 56;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub(super) usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub(super) usize);

impl PhysAddr {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new(addr: usize) -> Self {
        Self::try_new(addr).unwrap_or_else(|_| {
            panic!("Invalid physical address: {:#x}", addr);
        })
    }

    pub fn is_valid(addr: usize) -> bool {
        addr >> PA_WIDTH_SV39 == 0
    }

    #[inline]
    pub fn try_new(addr: usize) -> Result<Self, usize> {
        if Self::is_valid(addr) {
            Ok(Self(addr))
        } else {
            Err(addr)
        }
    }

    pub fn page_number(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> 12)
    }

    pub fn next_page_number(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) >> 12)
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl PhysPageNum {
    pub const fn new(ppn: usize) -> Self {
        Self(ppn)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa = PhysAddr::from(*self);
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa = PhysAddr::from(*self);
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa = PhysAddr::from(*self);
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysAddr({:#x})", self.0)
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysPageNum({:#x})", self.0)
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(ppn: PhysPageNum) -> Self {
        Self(ppn.0 << 12)
    }
}

#[cfg(test)]
mod tests {
    use core::usize;

    use super::*;

    #[test_case]
    fn test_phys_addr() {
        let valid_addrs = [
            (0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
            (0x0000_003F_FFFF_FFFF, 0x0000_0000_03FF_FFFF),
        ];
        for (addr, vpn) in valid_addrs {
            let phys_addr = PhysAddr::new(addr);
            assert_eq!(phys_addr.page_number().0, vpn);
            assert_eq!(phys_addr.0, addr);
        }

        let invalid_addrs = [0xFFFF_FFC0_0000_0123];
        for addr in invalid_addrs {
            assert!(!PhysAddr::is_valid(addr));
        }
    }
}
