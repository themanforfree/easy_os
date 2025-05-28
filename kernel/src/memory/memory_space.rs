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
    frame_allocator::{FRAME_ALLOCATOR, FrameAllocator},
    map_area::{MapArea, MapPermission, MapType},
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
    _areas: Vec<MapArea>,
}

impl MemorySpace {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            _areas: Vec::new(),
        }
    }

    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut space = Self::new_bare();
        // map trampoline
        space.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        space.map_range(
            stext as usize,
            etext as usize,
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        );
        println!("mapping .rodata section");
        space.map_range(
            srodata as usize,
            erodata as usize,
            MapType::Identical,
            MapPermission::R,
        );
        println!("mapping .data section");
        space.map_range(
            sdata as usize,
            edata as usize,
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        );
        println!("mapping .bss section");
        space.map_range(
            sbss_with_stack as usize,
            ebss as usize,
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        );
        println!("mapping physical memory");
        space.map_range(
            ekernel as usize,
            MEMORY_END,
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        );
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
        space
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::new(TRAMPOLINE).page_number(),
            PhysAddr::new(strampoline as usize).page_number(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    fn map_range(&mut self, start: usize, end: usize, map_type: MapType, perm: MapPermission) {
        let mut area = MapArea::new(VirtAddr::new(start), VirtAddr::new(end), map_type, perm);
        for vpn in area.range() {
            let ppn = match map_type {
                MapType::Identical => PhysPageNum::new(usize::from(vpn)),
                MapType::Framed => {
                    let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
                    let ppn = frame.ppn;
                    area.insert(vpn, frame);
                    ppn
                }
            };
            self.page_table
                .map(vpn, ppn, PTEFlags::from_bits(perm.bits()).unwrap());
        }
    }
}
