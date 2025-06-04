use alloc::{sync::Arc, vec::Vec};

use crate::{BLOCK_SIZE, BlockDevice, cache::get_block};

pub(crate) type DataBlock = [u8; BLOCK_SIZE];
pub(crate) type IndirectBlock = [u32; BLOCK_SIZE / 4];
pub(crate) type BitmapBlock = [u64; BLOCK_SIZE / 8];

/// The max number of direct blocks in an inode
const INODE_DIRECT_COUNT: usize = 28;
/// The number of indirect1 blocks in an inode
const INODE_INDIRECT1_COUNT: usize = BLOCK_SIZE / 4;
/// The number of indirect2 blocks in an inode
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
/// The upper bound of direct inode index
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
/// The upper bound of indirect1 inode index
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
/// The max length of inode name (including null terminator)
const NAME_LENGTH_LIMIT: usize = 28;

#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl SuperBlock {
    const MAGIC: u32 = u32::from_le_bytes([0x12, b'E', b'F', b'S']);

    /// Initialize a super block
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            magic: Self::MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    /// Check if a super block is valid using efs magic
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }
}

/// Type of a disk inode
#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}

#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    /// Initialize a disk inode, as well as all direct inodes under it
    /// indirect1 and indirect2 block are allocated only when they are needed
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }

    // pub fn is_file(&self) -> bool {
    //     self.type_ == DiskInodeType::File
    // }

    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    fn _data_blocks(size: u32) -> u32 {
        size.div_ceil(BLOCK_SIZE as u32)
    }

    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }

    pub fn _total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks;
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            // sub indirect1
            total += (data_blocks - INDIRECT1_BOUND).div_ceil(INODE_INDIRECT1_COUNT)
        }
        total as u32
    }

    pub fn total_blocks(&self) -> u32 {
        Self::_total_blocks(self.size)
    }

    /// Get the number of data blocks that have to be allocated given the new size of data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::_total_blocks(new_size) - self.total_blocks()
    }

    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < DIRECT_BOUND {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            get_block(self.indirect1 as usize, block_device)
                .lock()
                .read(0, |indirect: &IndirectBlock| {
                    indirect[inner_id - DIRECT_BOUND]
                })
        } else {
            let inner_id = inner_id - INDIRECT1_BOUND;
            let a0 = inner_id / INODE_INDIRECT1_COUNT;
            let b0 = inner_id % INODE_INDIRECT1_COUNT;
            get_block(self.indirect2 as usize, block_device)
                .lock()
                .read(0, |indirect: &IndirectBlock| {
                    get_block(indirect[a0] as usize, block_device)
                        .lock()
                        .read(0, |indirect: &IndirectBlock| indirect[b0])
                })
        }
    }

    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Arc<dyn BlockDevice>,
    ) {
        let mut current = self.data_blocks();
        self.size = new_size;
        let mut target = self.data_blocks();
        let mut new_blocks = new_blocks.into_iter();

        // fill direct
        while current < target.min(INODE_DIRECT_COUNT as u32) {
            self.direct[current as usize] = new_blocks.next().unwrap();
            current += 1;
        }

        // allocate indirect1
        if target > INODE_DIRECT_COUNT as u32 {
            if current == INODE_DIRECT_COUNT as u32 {
                // allocate indirect1 block
                self.indirect1 = new_blocks.next().unwrap();
            }
            // offset current and target
            current -= INODE_DIRECT_COUNT as u32;
            target -= INODE_DIRECT_COUNT as u32;
        } else {
            return;
        }
        // fill indirect1
        get_block(self.indirect1 as usize, block_device)
            .lock()
            .modify(0, |indirect: &mut IndirectBlock| {
                while current < target.min(INODE_INDIRECT1_COUNT as u32) {
                    indirect[current as usize] = new_blocks.next().unwrap();
                    current += 1;
                }
            });

        // allocate indirect2
        if target > INDIRECT1_BOUND as u32 {
            if current == INODE_INDIRECT1_COUNT as u32 {
                // allocate indirect2 block
                self.indirect2 = new_blocks.next().unwrap();
            }
            // offset current and target
            current -= INODE_INDIRECT1_COUNT as u32;
            target -= INODE_INDIRECT1_COUNT as u32;
        } else {
            return;
        }
        let mut a0 = current as usize / INODE_INDIRECT1_COUNT;
        let mut b0 = current as usize % INODE_INDIRECT1_COUNT;
        let a1 = target as usize / INODE_INDIRECT1_COUNT;
        let b1 = target as usize % INODE_INDIRECT1_COUNT;
        get_block(self.indirect2 as usize, block_device)
            .lock()
            .modify(0, |indirect2: &mut IndirectBlock| {
                while (a0 < a1) || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        indirect2[a0] = new_blocks.next().unwrap();
                    }
                    // fill current
                    get_block(indirect2[a0] as usize, block_device)
                        .lock()
                        .modify(0, |indirect1: &mut IndirectBlock| {
                            indirect1[b0] = new_blocks.next().unwrap();
                        });
                    // move to next
                    b0 += 1;
                    if b0 == INODE_INDIRECT1_COUNT {
                        b0 = 0;
                        a0 += 1;
                    }
                }
            });
    }

    /// Clear size to zero and return blocks that should be deallocated.
    /// We will clear the block contents to zero later.
    pub fn clear_size(&mut self, block_device: &Arc<dyn BlockDevice>) -> Vec<u32> {
        let mut cleared: Vec<u32> = Vec::new();
        let mut target = self.data_blocks() as usize;
        self.size = 0;
        let mut current = 0usize;
        // direct
        while current < target.min(INODE_DIRECT_COUNT) {
            cleared.push(self.direct[current]);
            self.direct[current] = 0;
            current += 1;
        }
        // indirect1 block
        if target > INODE_DIRECT_COUNT {
            cleared.push(self.indirect1);
            target -= INODE_DIRECT_COUNT;
            current = 0;
        } else {
            return cleared;
        }
        // indirect1
        get_block(self.indirect1 as usize, block_device)
            .lock()
            .modify(0, |indirect1: &mut IndirectBlock| {
                while current < target.min(INODE_INDIRECT1_COUNT) {
                    cleared.push(indirect1[current]);
                    //indirect1[current_blocks] = 0;
                    current += 1;
                }
            });
        self.indirect1 = 0;
        // indirect2 block
        if target > INODE_INDIRECT1_COUNT {
            cleared.push(self.indirect2);
            target -= INODE_INDIRECT1_COUNT;
        } else {
            return cleared;
        }
        // indirect2
        assert!(target <= INODE_INDIRECT2_COUNT);
        let a1 = target / INODE_INDIRECT1_COUNT;
        let b1 = target % INODE_INDIRECT1_COUNT;
        get_block(self.indirect2 as usize, block_device)
            .lock()
            .modify(0, |indirect2: &mut IndirectBlock| {
                // full indirect1 blocks
                for entry in indirect2.iter_mut().take(a1) {
                    cleared.push(*entry);
                    get_block(*entry as usize, block_device).lock().modify(
                        0,
                        |indirect1: &mut IndirectBlock| {
                            for entry in indirect1.iter() {
                                cleared.push(*entry);
                            }
                        },
                    );
                }
                // last indirect1 block
                if b1 > 0 {
                    cleared.push(indirect2[a1]);
                    get_block(indirect2[a1] as usize, block_device)
                        .lock()
                        .modify(0, |indirect1: &mut IndirectBlock| {
                            for entry in indirect1.iter().take(b1) {
                                cleared.push(*entry);
                            }
                        });
                    //indirect2[a1] = 0;
                }
            });
        self.indirect2 = 0;
        cleared
    }

    /// Read data from current disk inode
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if start >= end {
            return 0;
        }
        let mut start_block = start / BLOCK_SIZE;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];
            let blk = self.get_block_id(start_block as u32, block_device);
            get_block(blk as usize, block_device)
                .lock()
                .read(0, |data_block: &DataBlock| {
                    let src = &data_block[start % BLOCK_SIZE..start % BLOCK_SIZE + block_read_size];
                    dst.copy_from_slice(src);
                });
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }

    /// Write data into current disk inode
    /// size must be adjusted properly beforehand
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SIZE;
        let mut write_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let block_write_size = end_current_block - start;
            get_block(
                self.get_block_id(start_block as u32, block_device) as usize,
                block_device,
            )
            .lock()
            .modify(0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + block_write_size];
                let dst =
                    &mut data_block[start % BLOCK_SIZE..start % BLOCK_SIZE + block_write_size];
                dst.copy_from_slice(src);
            });
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }
}

/// A directory entry
#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT],
    inode_number: u32,
}

impl DirEntry {
    /// Create an empty directory entry
    pub fn empty() -> Self {
        Self {
            name: [0u8; NAME_LENGTH_LIMIT],
            inode_number: 0,
        }
    }

    /// Crate a directory entry from name and inode number
    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut bytes = [0u8; NAME_LENGTH_LIMIT];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            inode_number,
        }
    }

    /// Serialize into bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as usize as *const u8, size_of::<Self>())
        }
    }

    /// Serialize into mutable bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, size_of::<Self>())
        }
    }

    /// Get name of the entry
    pub fn name(&self) -> &str {
        let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }

    /// Get inode number of the entry
    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }
}
