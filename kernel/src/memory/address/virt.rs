//! Virtual Address Format
//! ```text
//! ┌─────────┬─────────────────────────────┬─────────────┐
//! │ SignExt │     Virtual Page Number     │ Page Offset │
//! ├─────────┼─────────┬─────────┬─────────┼─────────────┤
//! │ SignExt │ VPNp[2] │ VPNp[1] │ VPNp[0] │ Page Offset │
//! └─────────┴─────────┴─────────┴─────────┴─────────────┘
//!  63     39 38     30 29     21 20     12 11          0
//! ```
use core::{fmt::Debug, iter::Step};

use crate::config::PAGE_SIZE;

const VA_WIDTH_SV39: usize = 39;
const VA_WIDTH_SV39_MASK: usize = (1 << VA_WIDTH_SV39) - 1;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(usize);

impl VirtAddr {
    pub fn new(addr: usize) -> Self {
        Self::try_new(addr).unwrap_or_else(|_| {
            panic!("Invalid virtual address: {:#x}", addr);
        })
    }

    pub fn is_valid(addr: usize) -> bool {
        let upper_bits = addr >> (VA_WIDTH_SV39 - 1);
        upper_bits == 0 || upper_bits == usize::MAX >> (VA_WIDTH_SV39 - 1)
    }

    #[inline]
    pub fn try_new(addr: usize) -> Result<Self, usize> {
        if Self::is_valid(addr) {
            Ok(Self(addr))
        } else {
            Err(addr)
        }
    }

    pub fn page_number(&self) -> VirtPageNum {
        // Remove flag extension and offset
        VirtPageNum((self.0 & VA_WIDTH_SV39_MASK) >> 12)
    }

    pub fn next_page_number(&self) -> VirtPageNum {
        // Remove flag extension and offset
        VirtPageNum(((self.0 + PAGE_SIZE - 1) & VA_WIDTH_SV39_MASK) >> 12)
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtPageNum({:#x})", self.0)
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(vpn: VirtPageNum) -> Self {
        vpn.0
    }
}

impl Step for VirtPageNum {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        usize::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let end = usize::forward_checked(start.0, count)?;
        Some(Self(end))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let end = usize::backward_checked(start.0, count)?;
        Some(Self(end))
    }
}

#[cfg(test)]
mod tests {
    use core::usize;

    use super::*;

    #[test_case]
    fn test_virt_addr() {
        let valid_addrs = [
            (0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
            (0x0000_003F_FFFF_FFFF, 0x0000_0000_03FF_FFFF),
            (0xFFFF_FFC0_0000_0123, 0x0000_0000_0400_0000),
        ];
        for (addr, vpn) in valid_addrs {
            let virt_addr = VirtAddr::new(addr);
            assert_eq!(virt_addr.page_number().0, vpn);
            assert_eq!(virt_addr.0, addr);
        }

        let invalid_addrs = [0x0000_7FFF_FFFF_FFFF, 0x0000_0040_0000_0000];
        for addr in invalid_addrs {
            assert!(!VirtAddr::is_valid(addr));
        }
    }

    #[test_case]
    fn test_virt_range() {
        let start = VirtPageNum(1);
        let end = VirtPageNum(10);
        let mut n = 1;
        for i in start..=end {
            assert_eq!(i.0, n);
            n += 1;
        }
    }
}
