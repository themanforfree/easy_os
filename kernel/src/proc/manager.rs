use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use log::{info, trace};

use crate::{
    fs::{OpenFlags, open_file},
    sbi::shutdown,
    sync::UPSafeCell,
};

use super::{ProcControlBlock, ProcStatus, schedule, take_current_proc};

lazy_static! {
    /// A global instance of the process manager.
    pub static ref PROC_MANAGER: UPSafeCell<ProcManager> = unsafe { UPSafeCell::new(ProcManager::new()) };
    /// A global instance of the init process control block.``
    pub static ref INIT_PROC: Arc<ProcControlBlock> = Arc::new({
        let inode = open_file("init", OpenFlags::RDONLY).unwrap();
        ProcControlBlock::new(inode.read_all())
    });

}

const INIT_PROC_PID: usize = 0;

pub struct ProcManager {
    procs: VecDeque<Arc<ProcControlBlock>>,
}

impl ProcManager {
    fn new() -> Self {
        Self {
            procs: VecDeque::from([INIT_PROC.clone()]),
        }
    }

    pub fn push(&mut self, proc: Arc<ProcControlBlock>) {
        self.procs.push_back(proc);
    }

    pub fn pop(&mut self) -> Option<Arc<ProcControlBlock>> {
        self.procs.pop_front()
    }
}

pub fn suspend_current_and_run_next() {
    let proc = take_current_proc();
    let mut inner = proc.borrow_inner_mut();
    inner.status = ProcStatus::Ready;
    let ctx = &mut inner.ctx as *mut _;
    drop(inner);
    PROC_MANAGER.borrow_mut().push(proc);
    schedule(ctx);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let proc = take_current_proc();
    let pid = proc.pid();
    trace!("Process {pid} exits with exit code {exit_code}");

    if pid == INIT_PROC_PID {
        info!("Init process exits with exit code {exit_code}");
        if exit_code != 0 {
            shutdown(true)
        } else {
            shutdown(false)
        }
    }

    // update process data
    let mut inner = proc.borrow_inner_mut();
    inner.status = ProcStatus::Zombie;
    inner.exit_code = exit_code;

    // Move child processes to init process
    INIT_PROC.extend_children(inner.children.drain(..));
    debug_assert!(inner.children.is_empty());

    // Clear the memory space of the process, excluding the page table
    // TODO: should we free the page table?
    inner.memory_space.clear();
    let ctx = &mut inner.ctx as *mut _;
    drop(inner);
    drop(proc);
    schedule(ctx);
}
