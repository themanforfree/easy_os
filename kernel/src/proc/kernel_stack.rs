use crate::{
    config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE},
    memory::{KERNEL_SPACE, MapPermission, VirtAddr},
};

use super::pid::PidTracker;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub struct KernelStack {
    // pid: usize,
    kernel_stack_top: usize,
    // kernel_stack_bottom: usize,
}

impl KernelStack {
    pub fn new(pid: &PidTracker) -> Self {
        let pid = pid.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.borrow_mut().insert_framed_area(
            VirtAddr::new(kernel_stack_bottom),
            VirtAddr::new(kernel_stack_top),
            MapPermission::R | MapPermission::W,
        );
        Self {
            // pid,
            kernel_stack_top,
            // kernel_stack_bottom,
        }
    }

    pub fn get_top(&self) -> usize {
        self.kernel_stack_top
    }
}

// impl Drop for KernelStack {
//     fn drop(&mut self) {
//         let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
//         let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
//         KERNEL_SPACE
//             .borrow_mut()
//             .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
//     }
// }
