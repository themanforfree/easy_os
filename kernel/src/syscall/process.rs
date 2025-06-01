//! App management syscalls
use log::{info, trace};

use crate::{
    proc::{CPU, INIT_PROC, ProcStatus, schedule},
    sbi::shutdown,
};

const INIT_PROC_PID: usize = 0;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    let proc = CPU.borrow_mut().take_current().unwrap();
    let pid = proc.pid();
    trace!("Process {} exits with exit code {}", pid, exit_code);

    if pid == INIT_PROC_PID {
        info!("Init process exits with exit code {}", exit_code);
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
    schedule(ctx);

    unreachable!("Process {} should not return from sys_exit", pid);
}
