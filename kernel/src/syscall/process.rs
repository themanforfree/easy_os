//! App management syscalls
use log::info;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {exit_code}");
    // PROC_MANAGER.mark_current_exited();
    // PROC_MANAGER.run_next_task();
    todo!();
    // panic!("Unreachable in sys_exit!");
}
