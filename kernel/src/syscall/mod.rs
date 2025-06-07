use log::warn;
use syscall_id::*;

mod fs;
mod process;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> Option<isize> {
    let ret = match syscall_id {
        SYSCALL_OPEN => fs::sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => fs::sys_close(args[0]),
        SYSCALL_PIPE => fs::sys_pipe(args[0] as *mut usize),
        SYSCALL_READ => fs::sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => fs::sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => process::sys_exit(args[0] as i32),
        SYSCALL_YIELD => process::sys_yield(),
        SYSCALL_FORK => process::sys_fork(),
        SYSCALL_EXEC => process::sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_WAITPID => process::sys_waitpid(args[0] as isize, args[1] as *mut i32),
        _ => {
            warn!("Unknown syscall: {syscall_id}");
            return None;
        }
    };
    Some(ret)
}
