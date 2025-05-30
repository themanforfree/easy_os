global_asm!(include_str!("switch.S"));
use core::arch::global_asm;

use super::ProcContext;

unsafe extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub unsafe fn switch(
        current_task_cx_ptr: *mut ProcContext,
        next_task_cx_ptr: *const ProcContext,
    );
}
