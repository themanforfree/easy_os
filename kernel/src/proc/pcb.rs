use core::cell::RefMut;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    config::TRAP_FRAME,
    memory::{KERNEL_SPACE, MemorySpace, PhysPageNum, VirtAddr},
    proc::INIT_PROC,
    sync::UPSafeCell,
    trap::{TrapFrame, trap_handler},
};

use super::{
    ProcContext,
    kernel_stack::KernelStack,
    pid::{PID_ALLOCATOR, PidTracker},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcStatus {
    Ready,
    Running,
    // Waiting,
    Zombie,
    // Terminated,
}

pub struct ProcControlBlock {
    pid: PidTracker,
    kernel_stack: KernelStack,
    inner: UPSafeCell<ProcControlBlockInner>,
}

pub struct ProcControlBlockInner {
    pub ctx: ProcContext,
    pub status: ProcStatus,
    pub memory_space: MemorySpace,
    pub trap_frame_ppn: PhysPageNum,
    #[allow(dead_code)]
    pub base_size: usize,
    pub exit_code: i32,

    pub parent: Option<Weak<ProcControlBlock>>, // TODO: remove Option?
    pub children: Vec<Arc<ProcControlBlock>>,
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
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.get_top();
        let pcb = Self {
            pid,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcControlBlockInner {
                    status,
                    ctx: ProcContext::goto_trap_return(kernel_stack_top),
                    memory_space,
                    trap_frame_ppn,
                    base_size: user_sp,
                    exit_code: 0,
                    parent: None,
                    children: Vec::new(),
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

    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_space, user_sp, entry_point) = MemorySpace::from_elf(elf_data);
        let trap_frame_ppn = memory_space
            .translate(VirtAddr::new(TRAP_FRAME).page_number())
            .unwrap()
            .ppn();

        let mut inner = self.inner.borrow_mut();
        inner.memory_space = memory_space;
        inner.trap_frame_ppn = trap_frame_ppn;
        inner.base_size = user_sp;
        let trap_frame = inner.get_trap_frame_mut();
        *trap_frame = TrapFrame::new(
            entry_point,
            user_sp,
            KERNEL_SPACE.borrow_mut().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize, // physical address of trap handler
        );
    }

    pub fn borrow_inner_mut(&self) -> RefMut<'_, ProcControlBlockInner> {
        self.inner.borrow_mut()
    }

    pub fn pid(&self) -> usize {
        self.pid.0
    }

    pub fn extend_children(&self, children: impl Iterator<Item = Arc<ProcControlBlock>>) {
        assert_eq!(self.pid.0, 0, "Only init process can extend children");
        let mut inner = self.inner.borrow_mut();
        for child in children {
            child.borrow_inner_mut().parent = Some(Arc::downgrade(&INIT_PROC));
            inner.children.push(child);
        }
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent_inner = self.borrow_inner_mut();
        let child_space = parent_inner.memory_space.clone();
        let trap_frame_ppn = child_space
            .translate(VirtAddr::new(TRAP_FRAME).page_number())
            .unwrap()
            .ppn();
        let child_pid = PID_ALLOCATOR.borrow_mut().alloc();
        let kernel_stack = KernelStack::new(&child_pid);
        let kernel_stack_top = kernel_stack.get_top();
        let child_pcb = Arc::new(Self {
            pid: child_pid,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcControlBlockInner {
                    status: ProcStatus::Ready,
                    ctx: ProcContext::goto_trap_return(kernel_stack_top),
                    memory_space: child_space,
                    trap_frame_ppn,
                    base_size: parent_inner.base_size,
                    exit_code: 0,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                })
            },
        });
        parent_inner.children.push(Arc::clone(&child_pcb));

        let trap_frame = child_pcb.borrow_inner_mut().get_trap_frame_mut();
        trap_frame.kernel_sp = kernel_stack_top;

        child_pcb
    }
}

impl ProcControlBlockInner {
    pub fn get_token(&self) -> usize {
        self.memory_space.token()
    }

    pub fn get_trap_frame_mut(&self) -> &'static mut TrapFrame {
        self.trap_frame_ppn.get_mut()
    }

    pub fn is_zombie(&self) -> bool {
        self.status == ProcStatus::Zombie
    }
}
