use core::arch::{asm, global_asm};

use log::error;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
    utvec::TrapMode,
};
pub use trap_frame::TrapFrame;

use crate::{config::TRAMPOLINE, proc::PROC_MANAGER, timer::set_next_trigger};
use crate::{config::TRAP_FRAME, syscall::syscall};

mod trap_frame;

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `uservec`
pub fn init() {
    set_kernel_trap_entry();
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = PROC_MANAGER.get_current_trap_frame_mut();
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // 系统调用，恢复到用户态后不需要重复执行，将 sepc 加 4 设置为 ecall 之后的一条指令
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            error!(
                "PageFault in application, kernel killed it. fault_va = {:#x}, scause = {:?}",
                stval,
                scause.cause()
            );
            PROC_MANAGER.mark_current_exited();
            PROC_MANAGER.run_next_task();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("IllegalInstruction in application, kernel killed it.");
            PROC_MANAGER.mark_current_exited();
            PROC_MANAGER.run_next_task();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            PROC_MANAGER.mark_current_suspended();
            PROC_MANAGER.run_next_task();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// Return to user space after handling a trap
/// The First user process will call this function to enter to user space.
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_FRAME;
    let user_satp = PROC_MANAGER.get_current_token();
    unsafe extern "C" {
        /// The entry point for user space traps, which is the trampoline code.
        pub fn uservec();
        /// The return point for user space traps, which is the userret function.
        pub fn userret();
    }
    // userret and uservec art physical addresses, so we need to convert them to virtual addresses
    let restore_va = userret as usize - uservec as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i", // ensure instruction cache is flushed
            "jr {restore_va}", // call userret
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn),
        );
    }
}
