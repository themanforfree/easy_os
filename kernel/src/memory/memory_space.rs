use core::arch::asm;

use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use riscv::register::satp;
use xmas_elf::header::MAGIC;

use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_FRAME, USER_STACK_SIZE},
    sync::UPSafeCell,
};

use super::{
    PhysAddr, PhysPageNum, VirtAddr, VirtPageNum,
    frame_allocator::{FRAME_ALLOCATOR, FrameAllocator},
    map_area::{MapArea, MapPermission, MapType},
    page_table::{PTEFlags, PageTable, PageTableEntry},
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
    areas: Vec<MapArea>,
}

impl MemorySpace {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut space = Self::new_bare();
        // map trampoline
        space.map_trampoline();
        // map kernel sections
        let stext = stext as usize;
        let etext = etext as usize;
        let srodata = srodata as usize;
        let erodata = erodata as usize;
        let sdata = sdata as usize;
        let edata = edata as usize;
        let sbss_with_stack = sbss_with_stack as usize;
        let ebss = ebss as usize;
        let ekernel = ekernel as usize;
        println!(".text [{:#x}, {:#x})", stext, etext);
        println!(".rodata [{:#x}, {:#x})", srodata, erodata);
        println!(".data [{:#x}, {:#x})", sdata, edata);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack, ebss);
        println!("mapping .text section");
        space.map_range(
            VirtAddr::new(stext).page_number(),
            VirtAddr::new(etext).page_number(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        );
        println!("mapping .rodata section");
        space.map_range(
            VirtAddr::new(srodata).page_number(),
            VirtAddr::new(erodata).page_number(),
            MapType::Identical,
            MapPermission::R,
        );
        println!("mapping .data section");
        space.map_range(
            VirtAddr::new(sdata).page_number(),
            VirtAddr::new(edata).page_number(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        );
        println!("mapping .bss section");
        space.map_range(
            VirtAddr::new(sbss_with_stack).page_number(),
            VirtAddr::new(ebss).page_number(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        );
        println!("mapping physical memory");
        space.map_range(
            VirtAddr::new(ekernel).page_number(),
            VirtAddr::new(MEMORY_END).page_number(),
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

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.map_range(
            start_va.page_number(),
            end_va.page_number(),
            MapType::Framed,
            permission,
        );
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut space = Self::new_bare();
        space.map_trampoline();

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;

        let magic = elf_header.pt1.magic;
        assert_eq!(magic, MAGIC, "invalid elf!");

        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = None;
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if !ph
                .get_type()
                .is_ok_and(|t| t == xmas_elf::program::Type::Load)
            {
                continue;
            }

            let start_vpn = VirtAddr::new(ph.virtual_addr() as usize).page_number();
            let end_vpn =
                VirtAddr::new((ph.virtual_addr() + ph.mem_size()) as usize).next_page_number();
            let mut map_perm = MapPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                map_perm |= MapPermission::R;
            }
            if ph_flags.is_write() {
                map_perm |= MapPermission::W;
            }
            if ph_flags.is_execute() {
                map_perm |= MapPermission::X;
            }
            max_end_vpn = Some(end_vpn);
            let data = &elf_data[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            space.map_range_with_data(start_vpn, end_vpn, MapType::Framed, map_perm, data);
        }
        // map user stack with U flags
        let mut user_stack_bottom =
            VirtAddr::from(max_end_vpn.expect("No loadable program header found in ELF file"));
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        space.map_range(
            user_stack_bottom.page_number(),
            user_stack_top.page_number(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        // used in sbrk
        // FIXME: 这个有什么用？
        space.map_range(
            user_stack_top.page_number(),
            user_stack_top.page_number(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        // map TrapContext
        space.map_range(
            VirtAddr::new(TRAP_FRAME).page_number(),
            VirtAddr::new(TRAMPOLINE).page_number(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        );
        (
            space,
            user_stack_top.into(),
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn activate(&self) {
        let satp = self.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    fn map_trampoline(&mut self) {
        let vpn = VirtAddr::new(TRAMPOLINE).page_number();
        let ppn = PhysAddr::new(strampoline as usize).page_number();
        self.page_table.map(vpn, ppn, PTEFlags::R | PTEFlags::X);
    }

    fn map_range(
        &mut self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        map_type: MapType,
        perm: MapPermission,
    ) {
        self.map_range_with_data(start_vpn, end_vpn, map_type, perm, &[]);
    }

    fn map_range_with_data(
        &mut self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        map_type: MapType,
        perm: MapPermission,
        data: &[u8],
    ) {
        let mut area = MapArea::new(start_vpn, end_vpn, map_type, perm);
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
        if !data.is_empty() {
            self.page_table.copy_out(start_vpn, end_vpn, data);
        }
        self.areas.push(area);
    }
}
