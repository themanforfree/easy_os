#![no_std]
#![no_main]
#![feature(step_trait)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_utils::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(test, allow(unused))]

extern crate alloc;

use crate::common::clear_bss;
use crate::sbi::shutdown;
use core::arch::{global_asm, naked_asm};
use log::info;

#[macro_use]
mod console;
mod common;
mod config;
mod logger;
mod memory;
mod proc;
mod sbi;
mod sync;
mod syscall;
#[cfg(test)]
mod test_utils;
mod timer;
mod trap;

global_asm!(include_str!("link_app.S"));

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 16 * 1024; // 16 KiB

    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    naked_asm!(
    "la sp, {stack} + {stack_size}",
    "j  {main}",
    stack_size = const STACK_SIZE,
    stack      =   sym STACK,
    main       =   sym kernel_main,
    )
}

pub fn kernel_main(hart_id: usize, dtb_pa: usize) -> ! {
    clear_bss();
    logger::init();
    memory::init();
    trap::init();
    timer::init();
    proc::init();
    #[cfg(not(test))]
    {
        info!(r" _____         _     _  __                    _ ");
        info!(r"|_   _|__  ___| |_  | |/ /___ _ __ _ __   ___| |");
        info!(r"  | |/ _ \/ __| __| | ' // _ \ '__| '_ \ / _ \ |");
        info!(r"  | |  __/\__ \ |_  | . \  __/ |  | | | |  __/ |");
        info!(r"  |_|\___||___/\__| |_|\_\___|_|  |_| |_|\___|_|");
        info!(r"================================================");
        info!(r"| boot hart id          | {hart_id:20} |");
        info!(r"| dtb physical address  | {dtb_pa:#20x} |");
        info!(r"------------------------------------------------");
        info!("");
        proc::run();
    }
    #[cfg(test)]
    {
        info!("Running in test mode");
        info!("boot_hart_id: {}", hart_id);
        info!("dtb_pa: {:#x}", dtb_pa);
        test_main();
    }
    #[allow(unreachable_code)] // TODO: remove this
    shutdown(false);
}
