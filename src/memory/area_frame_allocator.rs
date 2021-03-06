use memory::{Frame, FrameAllocator, PhysAddr};
use multiboot2::{MemoryAreaIter, MemoryArea};
use arrayvec::ArrayVec;

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<Area>,
    areas: ArrayVec<[Area; 4]>,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

// 必须写这句，否则不能放在Mutex中？？？
unsafe impl Send for AreaFrameAllocator {}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame{ number: self.next_free_frame.number };

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.end_address() - 1;
                Frame::of_addr(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: PhysAddr, kernel_end: PhysAddr,
               multiboot_start: PhysAddr, multiboot_end: PhysAddr,
               memory_areas: MemoryAreaIter) -> AreaFrameAllocator
    {
        let areas: ArrayVec<[Area; 4]> = memory_areas.map(|a| Area::from(a)).collect();

        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::of_addr(0),
            current_area: None,
            areas,
            kernel_start: Frame::of_addr(kernel_start.0 as usize),
            kernel_end: Frame::of_addr(kernel_end.0 as usize),
            multiboot_start: Frame::of_addr(multiboot_start.0 as usize),
            multiboot_end: Frame::of_addr(multiboot_end.0 as usize),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.iter().filter(|area| {
            let address = area.end_address() - 1;
            Frame::of_addr(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.start_address())
            .map(|area| area.clone());

        if let Some(area) = self.current_area {
            let start_frame = Frame::of_addr(area.start_address());
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Area {
    start: usize,
    end: usize,
}

impl Area {
    pub fn start_address(&self) -> usize {
        self.start
    }
    pub fn end_address(&self) -> usize {
        self.end
    }
    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

impl<'a> From<&'a MemoryArea> for Area {
    fn from(a: &'a MemoryArea) -> Self {
        Area {
            start: a.start_address(),
            end: a.end_address(),
        }
    }
}