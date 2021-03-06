use super::address::PhysAddr;
use memory::FRAME_ALLOCATOR;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub(super) number: usize,
}

impl Frame {
    pub fn of_addr(address: usize) -> Frame {
        Frame{ number: address / PAGE_SIZE }
    }
    //TODO: Set private
    pub fn start_address(&self) -> PhysAddr {
        PhysAddr((self.number * PAGE_SIZE) as u64)
    }

    pub fn clone(&self) -> Frame {
        Frame { number: self.number }
    }
    //TODO: Set private    
    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start,
            end,
        }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
 }

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}