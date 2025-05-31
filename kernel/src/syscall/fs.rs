//! File and filesystem-related syscalls

use crate::{memory::PageTable, proc::CPU};

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let proc = CPU.borrow_mut().current();
            let token = proc.borrow_inner_mut().get_token();
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
