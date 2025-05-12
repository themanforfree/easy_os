use core::arch::global_asm;

use log::error;
use riscv::register::{
    scause::{self, Exception, Trap},
    stval, stvec,
    utvec::TrapMode,
};
pub use trap_frame::TrapFrame;

use crate::batch::run_next_app;
use crate::syscall::syscall;

mod trap_frame;

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    unsafe extern "C" {
        safe fn uservec();
    }
    unsafe {
        stvec::write(uservec as usize, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapFrame) -> &mut TrapFrame {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // 系统调用，恢复到用户态后不需要重复执行，将 sepc 加 4 设置为 ecall 之后的一条指令
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!("PageFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}
