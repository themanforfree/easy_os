use alloc::vec::Vec;
use sbi_rt::{NoReason, Shutdown, SystemFailure, system_reset};

use crate::proc::suspend_current_and_run_next;

pub fn shutdown(failure: bool) -> ! {
    if failure {
        system_reset(Shutdown, SystemFailure);
    } else {
        system_reset(Shutdown, NoReason);
    }
    unreachable!()
}

pub fn get_char_blocking() -> u8 {
    loop {
        #[allow(deprecated)] // TODO: do not use deprecated SBI calls
        let res = sbi_rt::legacy::console_getchar();
        if res != 0 {
            return (res & 0xFF) as u8;
        }
        suspend_current_and_run_next();
    }
}

pub fn read_chars_blocking(len: usize) -> Vec<u8> {
    let mut kernel_buffer = Vec::with_capacity(len);
    loop {
        let ch = get_char_blocking();
        kernel_buffer.push(ch);
        if kernel_buffer.len() >= len {
            break;
        }
    }
    kernel_buffer
}
