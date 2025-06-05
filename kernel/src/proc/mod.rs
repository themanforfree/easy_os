mod cpu;
mod ctx;
mod kernel_stack;
mod manager;
mod pcb;
mod pid;
mod switch;

use crate::fs::list_apps;

pub use self::cpu::{
    current_proc, current_token, current_trap_frame_mut, run, schedule, take_current_proc,
};
pub use self::ctx::ProcContext;
pub use self::manager::{
    INIT_PROC, PROC_MANAGER, exit_current_and_run_next, suspend_current_and_run_next,
};
pub use self::pcb::{ProcControlBlock, ProcStatus};
pub use self::switch::switch;

pub fn init() {
    list_apps();
}
