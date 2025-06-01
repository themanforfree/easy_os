mod cpu;
mod ctx;
mod loader;
mod manager;
mod pcb;
mod pid;
mod switch;

use loader::PROC_LOADER;

pub use self::cpu::{CPU, run, schedule};
pub use self::ctx::ProcContext;
pub use self::manager::{INIT_PROC, PROC_MANAGER};
pub use self::pcb::{ProcControlBlock, ProcStatus};
pub use self::switch::switch;

pub fn init() {
    PROC_LOADER.list_apps();
}
