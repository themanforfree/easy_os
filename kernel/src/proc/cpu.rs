use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::sync::UPSafeCell;

use super::{PROC_MANAGER, ProcContext, ProcControlBlock, ProcStatus, switch};

lazy_static! {
    /// A global instance of the CPU.
    static ref CPU: UPSafeCell<Cpu> = unsafe { UPSafeCell::new(Cpu::new()) };
}

struct Cpu {
    current: Option<Arc<ProcControlBlock>>,
    scheduler_ctx: ProcContext,
}

impl Cpu {
    fn new() -> Self {
        Self {
            current: None,
            scheduler_ctx: ProcContext::zero_init(),
        }
    }

    fn current(&self) -> Option<Arc<ProcControlBlock>> {
        self.current.clone()
    }

    fn take_current(&mut self) -> Option<Arc<ProcControlBlock>> {
        self.current.take()
    }

    fn set_current(&mut self, proc: Arc<ProcControlBlock>) {
        self.current = Some(proc);
    }

    fn get_scheduler_ctx_ptr(&mut self) -> *mut ProcContext {
        &mut self.scheduler_ctx as *mut _
    }
}

pub fn current_proc() -> Arc<ProcControlBlock> {
    CPU.borrow_mut().current().expect("No current process")
}

pub fn take_current_proc() -> Arc<ProcControlBlock> {
    CPU.borrow_mut().take_current().expect("No current process")
}

pub fn current_token() -> usize {
    current_proc().borrow_inner_mut().get_token()
}
pub fn current_trap_frame_mut() -> &'static mut crate::trap::TrapFrame {
    current_proc().borrow_inner_mut().get_trap_frame_mut()
}

pub fn run() -> ! {
    loop {
        let mut processor = CPU.borrow_mut();
        let proc_opt = PROC_MANAGER.borrow_mut().pop();
        if let Some(proc) = proc_opt {
            let scheduler_ctx = processor.get_scheduler_ctx_ptr();
            let mut proc_inner = proc.borrow_inner_mut();
            let proc_ctx = &proc_inner.ctx as *const _;
            proc_inner.status = ProcStatus::Running;
            drop(proc_inner);
            // release coming task TCB manually
            processor.set_current(proc);
            // release processor manually
            drop(processor);
            unsafe {
                switch(scheduler_ctx, proc_ctx);
            }
        }
    }
}

/// Switch the current process to the scheduler process.
pub fn schedule(switched_proc_ctx_ptr: *mut ProcContext) {
    // switched_proc_ctx_ptr is passed from outside, because generally,
    // proc data has been retrieved from outside, avoiding additional operations
    let scheduler_ctx_ptr = CPU.borrow_mut().get_scheduler_ctx_ptr();
    unsafe {
        switch(switched_proc_ctx_ptr, scheduler_ctx_ptr);
    }
}
