use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::sync::UPSafeCell;

use super::{ProcContext, ProcControlBlock};

lazy_static! {
    /// A global instance of the CPU.
    pub static ref CPU: UPSafeCell<Cpu> = unsafe { UPSafeCell::new(Cpu::new()) };
}

pub struct Cpu {
    current: Option<Arc<ProcControlBlock>>,
    scheduler_ctx: ProcContext,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            current: None,
            scheduler_ctx: ProcContext::zero_init(),
        }
    }

    pub fn current(&self) -> Arc<ProcControlBlock> {
        self.current.clone().unwrap()
    }

    pub fn set_current(&mut self, proc: Arc<ProcControlBlock>) {
        self.current = Some(proc);
    }

    pub fn get_scheduler_ctx_ptr(&mut self) -> *mut ProcContext {
        &mut self.scheduler_ctx as *mut _
    }
}
