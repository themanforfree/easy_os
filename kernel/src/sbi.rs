use sbi_rt::{NoReason, Shutdown, SystemFailure, system_reset};

pub fn shutdown(failure: bool) -> ! {
    if failure {
        system_reset(Shutdown, SystemFailure);
    } else {
        system_reset(Shutdown, NoReason);
    }
    unreachable!()
}
