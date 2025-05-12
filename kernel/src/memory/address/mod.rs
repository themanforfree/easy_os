//! Use SV39 architecture address space

mod phys;
mod virt;

pub use phys::{PhysAddr, PhysPageNum};
pub use virt::{VirtAddr, VirtPageNum};
