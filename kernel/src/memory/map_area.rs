use core::ops::Range;

use alloc::collections::btree_map::BTreeMap;

use bitflags::bitflags;

use super::{VirtAddr, VirtPageNum, frame_allocator::FrameTracker};

pub struct MapArea {
    start_vpn: VirtPageNum,
    end_vpn: VirtPageNum,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        Self {
            start_vpn: start_va.page_number(),
            end_vpn: end_va.page_number(),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub fn range(&self) -> Range<VirtPageNum> {
        self.start_vpn..self.end_vpn
    }

    pub fn insert(&mut self, vpn: VirtPageNum, frame: FrameTracker) {
        if self.map_type == MapType::Framed {
            self.data_frames.insert(vpn, frame);
        } else {
            panic!("Cannot insert frame into an identical map area");
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
