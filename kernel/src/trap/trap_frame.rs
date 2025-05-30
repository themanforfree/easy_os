use riscv::register::sstatus::{self, SPP, Sstatus};

/// Trap frame for user program
#[repr(C)]
pub struct TrapFrame {
    /// register x0~x31
    pub x: [usize; 32], // offset 0*8~31*8
    /// CSR sstatus
    pub sstatus: Sstatus, // offset 32*8
    /// saved user program counter
    pub sepc: usize, // offset 33*8
    /// the kernel satp, which is used to switch to kernel page table
    pub kernel_satp: usize, // offset 34*8
    /// the kernel stack pointer, which is used to switch to kernel stack
    pub kernel_sp: usize, // offset 35*8
    /// the trap handler address, which is used to handle traps in user mode
    pub trap_handler: usize, // offset 36*8
}

impl TrapFrame {
    /// create a new trap frame
    pub fn new(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut tf = Self {
            x: [0; 32],
            sepc: entry,
            sstatus,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        tf.set_sp(sp);
        tf
    }

    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
}
