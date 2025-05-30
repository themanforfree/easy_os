use crate::{
    config::{TRAP_FRAME, kernel_stack_position},
    memory::{KERNEL_SPACE, MapPermission, MemorySpace, PhysPageNum, VirtAddr},
    trap::{TrapFrame, trap_handler},
};

use super::ProcContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcStatus {
    Ready,
    Running,
    // Waiting,
    Terminated,
}

pub struct ProcessControlBlock {
    pub cx: ProcContext,
    pub status: ProcStatus,
    pub memory_space: MemorySpace,
    pub trap_frame_ppn: PhysPageNum,
    #[allow(unused)] // TODO: remove this when not needed
    pub base_size: usize,
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8], pid: usize) -> Self {
        let (memory_space, user_sp, entry_point) = MemorySpace::from_elf(elf_data);
        let trap_frame_ppn = memory_space
            .translate(VirtAddr::new(TRAP_FRAME).page_number())
            .unwrap()
            .ppn();
        let status = ProcStatus::Ready;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.borrow_mut().insert_framed_area(
            VirtAddr::new(kernel_stack_bottom),
            VirtAddr::new(kernel_stack_top),
            MapPermission::R | MapPermission::W,
        );
        let pcb = Self {
            status,
            cx: ProcContext::goto_trap_return(kernel_stack_top),
            memory_space,
            trap_frame_ppn,
            base_size: user_sp,
        };
        let trap_frame = pcb.get_trap_frame_mut();
        *trap_frame = TrapFrame::new(
            entry_point,
            user_sp,
            KERNEL_SPACE.borrow_mut().token(),
            kernel_stack_top,
            trap_handler as usize, // physical address of trap handler
        );
        pcb
    }

    pub fn get_token(&self) -> usize {
        self.memory_space.token()
    }

    pub fn get_trap_frame_mut(&self) -> &'static mut TrapFrame {
        self.trap_frame_ppn.get_mut()
    }
}
