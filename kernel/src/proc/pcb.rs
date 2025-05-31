use core::cell::RefMut;

use crate::{
    config::{TRAP_FRAME, kernel_stack_position},
    memory::{KERNEL_SPACE, MapPermission, MemorySpace, PhysPageNum, VirtAddr},
    sync::UPSafeCell,
    trap::{TrapFrame, trap_handler},
};

use super::{ProcContext, pid::PID_ALLOCATOR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcStatus {
    Ready,
    Running,
    // Waiting,
    // Terminated,
}

pub struct ProcControlBlock {
    inner: UPSafeCell<ProcControlBlockInner>,
}

pub struct ProcControlBlockInner {
    pub ctx: ProcContext,
    pub status: ProcStatus,
    pub memory_space: MemorySpace,
    pub trap_frame_ppn: PhysPageNum,
    #[allow(dead_code)]
    pub base_size: usize,
}

impl ProcControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_space, user_sp, entry_point) = MemorySpace::from_elf(elf_data);
        let trap_frame_ppn = memory_space
            .translate(VirtAddr::new(TRAP_FRAME).page_number())
            .unwrap()
            .ppn();
        let status = ProcStatus::Ready;
        let pid = PID_ALLOCATOR.borrow_mut().alloc();
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid.0);
        KERNEL_SPACE.borrow_mut().insert_framed_area(
            VirtAddr::new(kernel_stack_bottom),
            VirtAddr::new(kernel_stack_top),
            MapPermission::R | MapPermission::W,
        );
        let pcb = Self {
            inner: unsafe {
                UPSafeCell::new(ProcControlBlockInner {
                    status,
                    ctx: ProcContext::goto_trap_return(kernel_stack_top),
                    memory_space,
                    trap_frame_ppn,
                    base_size: user_sp,
                })
            },
        };
        let trap_frame = pcb.inner.borrow_mut().get_trap_frame_mut();
        *trap_frame = TrapFrame::new(
            entry_point,
            user_sp,
            KERNEL_SPACE.borrow_mut().token(),
            kernel_stack_top,
            trap_handler as usize, // physical address of trap handler
        );
        pcb
    }

    pub fn borrow_inner_mut(&self) -> RefMut<'_, ProcControlBlockInner> {
        self.inner.borrow_mut()
    }
}

impl ProcControlBlockInner {
    pub fn get_token(&self) -> usize {
        self.memory_space.token()
    }

    pub fn get_trap_frame_mut(&self) -> &'static mut TrapFrame {
        self.trap_frame_ppn.get_mut()
    }

    pub fn get_ctx_ptr(&self) -> *const ProcContext {
        &self.ctx as *const _
    }
}
