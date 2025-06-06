//! RISC-V timer-related functionality

use crate::config::CLOCK_FREQ;
use riscv::register::{sie, time};
use sbi_rt::set_timer;

const TICKS_PER_SEC: usize = 100;
// const MSEC_PER_SEC: usize = 1000;

pub fn get_time() -> usize {
    time::read()
}

// /// get current time in microseconds
// pub fn get_time_ms() -> usize {
//     time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
// }

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer((get_time() + CLOCK_FREQ / TICKS_PER_SEC) as u64);
}

pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_trigger();
}
