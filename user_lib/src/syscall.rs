use syscall_id::*;

macro_rules! syscall {
    ($id:expr $(, $arg:expr)* ) => {{
        let mut args = [0usize; 3];
        let _arg_slice = [$($arg as usize),*];
        for i in 0..3.min(_arg_slice.len()) {
            args[i] = _arg_slice[i];
        }
        let mut ret: isize;
        unsafe {
            core::arch::asm!(
                "ecall",
                inlateout("x10") args[0] => ret,
                in("x11") args[1],
                in("x12") args[2],
                in("x17") $id,
            );
        }
        ret
    }};
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall!(SYSCALL_OPEN, path.as_ptr() as usize, flags)
}

pub fn sys_close(fd: usize) -> isize {
    syscall!(SYSCALL_CLOSE, fd)
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall!(SYSCALL_READ, fd, buffer.as_mut_ptr() as usize, buffer.len())
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall!(SYSCALL_WRITE, fd, buffer.as_ptr() as usize, buffer.len())
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall!(SYSCALL_EXIT, exit_code as usize)
}

pub fn sys_fork() -> isize {
    syscall!(SYSCALL_FORK)
}

pub fn sys_waitpid(pid: isize, status: *mut i32) -> isize {
    syscall!(SYSCALL_WAITPID, pid as usize, status as usize)
}

pub fn sys_yield() -> isize {
    syscall!(SYSCALL_YIELD)
}

pub fn sys_exec(path: &str) -> isize {
    syscall!(SYSCALL_EXEC, path.as_ptr() as usize)
}
