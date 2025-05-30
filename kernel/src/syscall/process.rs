//! App management syscalls
use log::info;

use crate::proc::PROC_MANAGER;
/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {exit_code}");
    PROC_MANAGER.mark_current_exited();
    PROC_MANAGER.run_next_task();
    panic!("Unreachable in sys_exit!");
}
