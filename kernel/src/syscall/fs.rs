//! File and filesystem-related syscalls

use crate::{memory::PageTable, proc::PROC_MANAGER};

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let token = PROC_MANAGER.get_current_token();
            let pt = PageTable::from_token(token);
            let slice = pt.copy_in(buf, len);
            let str = core::str::from_utf8(&slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
