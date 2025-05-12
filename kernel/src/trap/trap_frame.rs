use riscv::register::sstatus::{self, SPP, Sstatus};

///! Trap frame for user program
pub struct TrapFrame {
    /// register x0~x31
    pub x: [usize; 32], // offset 0*8~31*8
    /// CSR sstatus
    #[allow(unused)] // TODO remove this when sstatus is used
    pub sstatus: Sstatus, // offset 32*8
    /// saved user program counter
    pub sepc: usize, // offset 33*8
}

impl TrapFrame {
    /// create a new trap frame
    pub fn new(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut tf = Self {
            x: [0; 32],
            sepc: entry,
            sstatus,
        };
        tf.set_sp(sp);
        tf
    }

    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
}
