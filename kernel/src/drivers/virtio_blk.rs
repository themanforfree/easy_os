use alloc::vec::Vec;
use easy_fs::BlockDevice;
use lazy_static::*;
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

use crate::memory::{
    FRAME_ALLOCATOR, FrameAllocator, FrameTracker, KERNEL_SPACE, PageTable, PhysAddr, PhysPageNum,
    VirtAddr,
};
use crate::sync::UPSafeCell;

const VIRTIO0: usize = 0x10001000;
pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, VirtioHal>>);

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .borrow_mut()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .borrow_mut()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        }
    }
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let mut ppn_base = PhysPageNum::new(0);
        for i in 0..pages {
            let frame = FRAME_ALLOCATOR.borrow_mut().frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn, ppn_base + i);
            QUEUE_FRAMES.borrow_mut().push(frame);
        }
        let pa = PhysAddr::from(ppn_base);
        pa.into()
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let ppn_base = PhysAddr::from(pa).page_number();
        for ppn_base in ppn_base..ppn_base + pages {
            FRAME_ALLOCATOR.borrow_mut().frame_dealloc(ppn_base);
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(addr: usize) -> usize {
        PageTable::from_token(KERNEL_SPACE.borrow_mut().token())
            .translate_va(VirtAddr::from(addr))
            .unwrap()
            .into()
    }
}
