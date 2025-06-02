//! App management syscalls
use alloc::sync::Arc;
use log::{trace, warn};

use crate::proc::{
    PROC_LOADER, PROC_MANAGER, current_proc, exit_current_and_run_next,
    suspend_current_and_run_next,
};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("sys_exit: exit_code = {exit_code}");
    exit_current_and_run_next(exit_code);
    unreachable!("Process should not return from sys_exit");
}

pub fn sys_yield() -> isize {
    trace!("sys_yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_fork() -> isize {
    trace!("sys_fork");
    let parent = current_proc();
    let child = parent.fork();

    let child_pid = child.pid();
    let child_trap_frame = child.borrow_inner_mut().get_trap_frame_mut();
    child_trap_frame.x[10] = 0;

    PROC_MANAGER.borrow_mut().push(child);

    child_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let proc = current_proc();
    let name = proc
        .borrow_inner_mut()
        .memory_space
        .read_c_str(path)
        .unwrap();
    trace!("sys_exec: path = {name}");
    if let Some(elf_data) = PROC_LOADER.get_app_data_by_name(&name) {
        proc.exec(elf_data);
        0
    } else {
        warn!("sys_exec: app {name} not found");
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, status: *mut i32) -> isize {
    let proc = current_proc();
    let mut proc_inner = proc.borrow_inner_mut();

    let Some(idx) = proc_inner
        .children
        .iter()
        .enumerate()
        .find_map(|(idx, pcb)| (pid == -1 || pcb.pid() == pid as usize).then_some(idx))
    else {
        // No child process matches the given pid
        return -1;
    };
    if !proc_inner.children[idx].borrow_inner_mut().is_zombie() {
        return -2; // Child process is still running
    }

    let child = proc_inner.children.remove(idx);
    assert_eq!(Arc::strong_count(&child), 1);
    let proc_pid = child.pid();
    let exit_code = child.borrow_inner_mut().exit_code;
    *proc_inner.memory_space.translated_mut_ptr(status) = exit_code;
    proc_pid as isize
}
