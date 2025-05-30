use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::info;

use crate::{
    proc::{ProcContext, ProcStatus, switch},
    sbi::shutdown,
    sync::UPSafeCell,
    trap::TrapFrame,
};

use super::ProcessControlBlock;

lazy_static! {
    /// A global instance of the process manager.
    pub static ref PROC_MANAGER: ProcManager = ProcManager::init();
}

struct ProcManagerInner {
    /// process list
    procs: Vec<ProcessControlBlock>,
    /// id of current `Running` process
    current_proc: usize,
}

pub struct ProcManager {
    /// total number of processes
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<ProcManagerInner>,
}

unsafe extern "C" {
    safe fn _num_apps();
}

impl ProcManager {
    /// get applications data
    pub fn get_app_data(app_id: usize) -> &'static [u8] {
        let num_app_addr = _num_apps as usize;
        let num_app_ptr = num_app_addr as *const usize;
        let num_app = unsafe { num_app_ptr.read_volatile() };
        let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
        assert!(app_id < num_app);
        unsafe {
            core::slice::from_raw_parts(
                app_start[app_id] as *const u8,
                app_start[app_id + 1] - app_start[app_id],
            )
        }
    }

    fn init() -> Self {
        info!("Initializing process manager...");
        let num_app_addr = _num_apps as usize;
        let num_app = unsafe { (num_app_addr as *const usize).read_volatile() };
        info!("Total number of applications: {num_app}");
        let mut procs = Vec::with_capacity(num_app);
        for i in 0..num_app {
            info!("Loading application {i}");
            let elf_data = Self::get_app_data(i);
            procs.push(ProcessControlBlock::new(elf_data, i));
            info!("Loaded application {i}");
        }

        Self {
            num_app,
            inner: unsafe {
                UPSafeCell::new(ProcManagerInner {
                    procs,
                    current_proc: 0,
                })
            },
        }
    }

    pub fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow_mut();
        inner.procs[inner.current_proc].get_token()
    }

    pub fn get_current_trap_frame_mut(&self) -> &'static mut TrapFrame {
        let inner = self.inner.borrow_mut();
        inner.procs[inner.current_proc].get_trap_frame_mut()
    }

    pub fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let cur = inner.current_proc;
        inner.procs[cur].status = ProcStatus::Terminated;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow_mut();
        let current = inner.current_proc;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.procs[*id].status == ProcStatus::Ready)
    }

    pub fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_proc;
            inner.procs[next].status = ProcStatus::Running;
            inner.current_proc = next;
            let current_proc_cx_ptr = &mut inner.procs[current].cx as *mut ProcContext;
            let next_proc_cx_ptr = &inner.procs[next].cx as *const ProcContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                switch(current_proc_cx_ptr, next_proc_cx_ptr);
            }
            // go back to user mode
        } else {
            info!("All applications completed!");
            shutdown(false);
        }
    }

    pub fn run_first_proc(&self) -> ! {
        let mut inner = self.inner.borrow_mut();
        let next_proc = &mut inner.procs[0];
        next_proc.status = ProcStatus::Running;
        let next_proc_cx_ptr = &next_proc.cx as *const ProcContext;
        drop(inner);
        let mut _unused = ProcContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe { switch(&mut _unused as *mut _, next_proc_cx_ptr) };
        panic!("unreachable in run_first_proc!");
    }
}
