use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use log::info;

use crate::sync::UPSafeCell;

use super::ProcControlBlock;

lazy_static! {
    /// A global instance of the process manager.
    pub static ref PROC_MANAGER: UPSafeCell<ProcManager> = unsafe { UPSafeCell::new(ProcManager::new()) };
}

pub struct ProcManager {
    procs: VecDeque<Arc<ProcControlBlock>>,
}

impl ProcManager {
    fn new() -> Self {
        info!("Initializing process manager...");
        Self {
            procs: VecDeque::new(),
        }
    }

    pub fn push(&mut self, proc: Arc<ProcControlBlock>) {
        self.procs.push_back(proc);
    }

    pub fn pop(&mut self) -> Option<Arc<ProcControlBlock>> {
        self.procs.pop_front()
    }

    // pub fn get_current_token(&self) -> usize {
    //     // let inner = self.inner.borrow_mut();
    //     // inner.procs[inner.current_proc].get_token()
    // }

    // pub fn get_current_trap_frame_mut(&self) -> &'static mut TrapFrame {
    //     let inner = self.inner.borrow_mut();
    //     inner.procs[inner.current_proc].get_trap_frame_mut()
    // }

    // pub fn mark_current_exited(&self) {
    //     let mut inner = self.inner.borrow_mut();
    //     let cur = inner.current_proc;
    //     inner.procs[cur].status = ProcStatus::Terminated;
    // }

    // pub fn mark_current_suspended(&self) {
    //     let mut inner = self.inner.borrow_mut();
    //     let cur = inner.current_proc;
    //     inner.procs[cur].status = ProcStatus::Ready;
    // }

    // fn find_next_task(&self) -> Option<usize> {
    //     let inner = self.inner.borrow_mut();
    //     let current = inner.current_proc;
    //     (current + 1..current + self.num_app + 1)
    //         .map(|id| id % self.num_app)
    //         .find(|id| inner.procs[*id].status == ProcStatus::Ready)
    // }

    // pub fn run_next_task(&self) {
    //     if let Some(next) = self.find_next_task() {
    //         let mut inner = self.inner.borrow_mut();
    //         let current = inner.current_proc;
    //         inner.procs[next].status = ProcStatus::Running;
    //         inner.current_proc = next;
    //         let current_proc_cx_ptr = &mut inner.procs[current].cx as *mut ProcContext;
    //         let next_proc_cx_ptr = &inner.procs[next].cx as *const ProcContext;
    //         drop(inner);
    //         // before this, we should drop local variables that must be dropped manually
    //         unsafe {
    //             switch(current_proc_cx_ptr, next_proc_cx_ptr);
    //         }
    //         // go back to user mode
    //     } else {
    //         info!("All applications completed!");
    //         shutdown(false);
    //     }
    // }

    // pub fn run_first_proc(&self) -> ! {
    //     let mut inner = self.inner.borrow_mut();
    //     let next_proc = &mut inner.procs[0];
    //     next_proc.status = ProcStatus::Running;
    //     let next_proc_cx_ptr = &next_proc.cx as *const ProcContext;
    //     drop(inner);
    //     let mut _unused = ProcContext::zero_init();
    //     // before this, we should drop local variables that must be dropped manually
    //     unsafe { switch(&mut _unused as *mut _, next_proc_cx_ptr) };
    //     panic!("unreachable in run_first_proc!");
    // }
}
