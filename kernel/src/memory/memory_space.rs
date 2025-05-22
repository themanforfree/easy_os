use core::arch::asm;

use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use riscv::register::satp;

use crate::{
    config::{MEMORY_END, TRAMPOLINE},
    sync::UPSafeCell,
};

use super::{
    PhysAddr, PhysPageNum, VirtAddr,
    frame_allocator::FrameTracker,
    page_table::{PTEFlags, PageTable},
};

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySpace>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySpace::new_kernel()) });
}

unsafe extern "C" {
    safe fn stext();
    safe fn etext();
    safe fn srodata();
    safe fn erodata();
    safe fn sdata();
    safe fn edata();
    safe fn sbss_with_stack();
    safe fn ebss();
    safe fn ekernel();
    safe fn strampoline();
}

pub struct MemorySpace {
    page_table: PageTable,
    pages: Vec<FrameTracker>,
}

impl MemorySpace {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            pages: Vec::new(),
        }
    }

    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::new(TRAMPOLINE).page_number(),
            PhysAddr::new(strampoline as usize).page_number(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    // TODO: support framed,current only support identical
    fn map_range(&mut self, start: usize, end: usize, flags: PTEFlags) {
        let start = VirtAddr::new(start).page_number();
        let end = VirtAddr::new(end).page_number();
        for vpn in start..end {
            let ppn = PhysPageNum::new(usize::from(vpn));
            self.page_table.map(vpn, ppn, flags);
        }
    }

    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memory_set.map_range(stext as usize, etext as usize, PTEFlags::R | PTEFlags::X);
        println!("mapping .rodata section");
        memory_set.map_range(srodata as usize, erodata as usize, PTEFlags::R);
        println!("mapping .data section");
        memory_set.map_range(sdata as usize, edata as usize, PTEFlags::R | PTEFlags::W);
        println!("mapping .bss section");
        memory_set.map_range(
            sbss_with_stack as usize,
            ebss as usize,
            PTEFlags::R | PTEFlags::W,
        );
        println!("mapping physical memory");
        memory_set.map_range(ekernel as usize, MEMORY_END, PTEFlags::R | PTEFlags::W);
        // println!("mapping memory-mapped registers");
        // for pair in MMIO {
        //     memory_set.push(
        //         MapArea::new(
        //             (*pair).0.into(),
        //             ((*pair).0 + (*pair).1).into(),
        //             MapType::Identical,
        //             MapPermission::R | MapPermission::W,
        //         ),
        //         None,
        //     );
        // }
        memory_set
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
}
