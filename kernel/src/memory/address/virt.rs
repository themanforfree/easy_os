//! Virtual Address Format
//! ```text
//! ┌─────────┬─────────────────────────────┬─────────────┐
//! │ SignExt │     Virtual Page Number     │ Page Offset │
//! ├─────────┼─────────┬─────────┬─────────┼─────────────┤
//! │ SignExt │ VPNp[2] │ VPNp[1] │ VPNp[0] │ Page Offset │
//! └─────────┴─────────┴─────────┴─────────┴─────────────┘
//!  63     39 38     30 29     21 20     12 11          0
//! ```
use core::fmt::Debug;

const VA_WIDTH_SV39: usize = 39;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(usize);

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
        VirtPageNum((self.0 & (1 << VA_WIDTH_SV39) - 1) >> 12)
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
}
