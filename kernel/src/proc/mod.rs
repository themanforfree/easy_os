mod cpu;
mod ctx;
mod loader;
mod manager;
mod pcb;
mod pid;
mod switch;

use alloc::sync::Arc;
use loader::PROC_LOADER;

pub use self::cpu::CPU;
pub use self::ctx::ProcContext;
pub use self::manager::PROC_MANAGER;
pub use self::pcb::{ProcControlBlock, ProcStatus};
pub use self::switch::switch;

pub fn init() {
    PROC_LOADER.list_apps();
    let init_proc_data = PROC_LOADER.get_app_data_by_name("init").unwrap();
    let pcb = ProcControlBlock::new(init_proc_data);
    PROC_MANAGER.borrow_mut().push(Arc::new(pcb));
}

pub fn run() {
    loop {
        let mut processor = CPU.borrow_mut();
        if let Some(proc) = PROC_MANAGER.borrow_mut().pop() {
            let scheduler_ctx = processor.get_scheduler_ctx_ptr();
            let mut proc_inner = proc.borrow_inner_mut();
            let proc_ctx = proc_inner.get_ctx_ptr();
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
