pub use self::area_frame_allocator::AreaFrameAllocator;
pub use arch::paging::*;
pub use self::stack_allocator::Stack;
pub use self::address::*;
pub use self::frame::*;
pub use self::memory_set::*;

use multiboot2::BootInformation;
use consts::KERNEL_OFFSET;
use arch::paging;
use spin::Mutex;
use super::HEAP_ALLOCATOR;

mod memory_set;
mod area_frame_allocator;
pub mod heap_allocator;
mod stack_allocator;
mod address;
mod frame;

pub static FRAME_ALLOCATOR: Mutex<Option<AreaFrameAllocator>> = Mutex::new(None);

pub fn alloc_frame() -> Frame {
    FRAME_ALLOCATOR.lock()
        .as_mut().expect("frame allocator is not initialized")
        .allocate_frame().expect("no more frame")
}

// Return true to continue, false to halt
pub fn page_fault_handler(addr: VirtAddr) -> bool {
    // Handle copy on write
    let mut page_table = unsafe { ActivePageTable::new() };
    page_table.try_copy_on_write(addr)
}

pub fn init(boot_info: BootInformation) -> MemoryController {
    assert_has_not_been_called!("memory::init must be called only once");

    info!("{:?}", boot_info);

    let memory_map_tag = boot_info.memory_map_tag().expect(
        "Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect(
        "Elf sections tag required");

    let kernel_start = PhysAddr(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.start_address()).min().unwrap());
    let kernel_end = PhysAddr::from_kernel_virtual(elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.end_address()).max().unwrap() as usize);

    let boot_info_start = PhysAddr(boot_info.start_address() as u64);
    let boot_info_end = PhysAddr(boot_info.end_address() as u64);

    *FRAME_ALLOCATOR.lock() = Some(AreaFrameAllocator::new(
        kernel_start, kernel_end,
        boot_info_start, boot_info_end,
        memory_map_tag.memory_areas()
    ));

    let kernel_stack = remap_the_kernel(boot_info);

    use self::paging::Page;
    use consts::{KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE};

    unsafe { HEAP_ALLOCATOR.lock().init(KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE); }

    let stack_allocator = {
        let stack_alloc_range = Page::range_of(KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE,
                                               KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE + 0x1000000);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };
    
    MemoryController {
        kernel_stack: Some(kernel_stack),
        active_table: unsafe { ActivePageTable::new() },
        stack_allocator,
    }
}

pub fn remap_the_kernel(boot_info: BootInformation) -> Stack {
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut memory_set = MemorySet::from(boot_info.elf_sections_tag().unwrap());

    use consts::{KERNEL_OFFSET, KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE};
    memory_set.push(MemoryArea::new_kernel(KERNEL_OFFSET + 0xb8000, KERNEL_OFFSET + 0xb9000, EntryFlags::WRITABLE, "VGA"));
    memory_set.push(MemoryArea::new(KERNEL_HEAP_OFFSET, KERNEL_HEAP_OFFSET + KERNEL_HEAP_SIZE, EntryFlags::WRITABLE, "kernel_heap"));

    let mut page_table = InactivePageTable::new(alloc_frame(), &mut active_table);
    active_table.with(&mut page_table, |pt| memory_set.map(pt));
    debug!("{:#x?}", memory_set);

    let old_table = active_table.switch(page_table);
    info!("NEW TABLE!!!");

    // turn the stack bottom into a guard page
    extern { fn stack_bottom(); }
    let stack_bottom = PhysAddr(stack_bottom as u64).to_kernel_virtual();
    let stack_bottom_page = Page::of_addr(stack_bottom);
    active_table.unmap(stack_bottom_page);
    let kernel_stack = Stack::new(stack_bottom + 8 * PAGE_SIZE, stack_bottom + 1 * PAGE_SIZE);
    debug!("guard page at {:#x}", stack_bottom_page.start_address());

    kernel_stack
}

use multiboot2::{ElfSectionsTag, ElfSection, ElfSectionFlags};

impl From<ElfSectionsTag> for MemorySet {
    fn from(sections: ElfSectionsTag) -> Self {
        assert_has_not_been_called!();
        // WARNING: must ensure it's large enough
        static mut SPACE: [u8; 0x1000] = [0; 0x1000];
        let mut set = unsafe { MemorySet::new_from_raw_space(&mut SPACE) };
        for section in sections.sections() {
            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }
            set.push(MemoryArea::from(section));
        }
        set
    }
}

impl From<ElfSection> for MemoryArea {
    fn from(section: ElfSection) -> Self {
        use self::address::FromToVirtualAddress;
        let mut start_addr = section.start_address() as usize;
        let mut end_addr = section.end_address() as usize;
        assert_eq!(start_addr % PAGE_SIZE, 0, "sections need to be page aligned");
        let name = unsafe { &*(section.name() as *const str) };
        if start_addr < KERNEL_OFFSET {
            start_addr += KERNEL_OFFSET;
            end_addr += KERNEL_OFFSET;
        }
        MemoryArea::new_kernel(start_addr, end_addr, EntryFlags::from(section.flags()), name)
    }
}

impl From<ElfSectionFlags> for EntryFlags {
    fn from(elf_flags: ElfSectionFlags) -> Self {
        let mut flags = EntryFlags::empty();

        if elf_flags.contains(ElfSectionFlags::ALLOCATED) {
            // section is loaded to memory
            flags = flags | EntryFlags::PRESENT;
        }
        if elf_flags.contains(ElfSectionFlags::WRITABLE) {
            flags = flags | EntryFlags::WRITABLE;
        }
        // if !elf_flags.contains(ElfSectionFlags::EXECUTABLE) {
        //     flags = flags | EntryFlags::NO_EXECUTE;
        // }
        flags = flags & !EntryFlags::NO_EXECUTE;
        flags
    }
}

pub struct MemoryController {
    pub kernel_stack: Option<Stack>,
    active_table: paging::ActivePageTable,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController { ref mut kernel_stack,
                                    ref mut active_table,
                                    ref mut stack_allocator } = self;
        stack_allocator.alloc_stack(active_table, size_in_pages)
    }
    pub fn new_page_table(&mut self) -> InactivePageTable {
        let frame = alloc_frame();
        let page_table = InactivePageTable::new(frame, &mut self.active_table);
        page_table
    }
    pub fn map_page_identity(&mut self, addr: usize) {
        let frame = Frame::of_addr(addr);
        let flags = EntryFlags::WRITABLE;
        self.active_table.identity_map(frame, flags);
    }
    pub fn map_page_p2v(&mut self, addr: PhysAddr) {
        let page = Page::of_addr(addr.to_kernel_virtual());
        let frame = Frame::of_addr(addr.get());
        let flags = EntryFlags::WRITABLE;
        self.active_table.map_to(page, frame, flags);
    }
    pub fn make_page_table(&mut self, set: &MemorySet) -> InactivePageTable {
        let mut page_table = InactivePageTable::new(alloc_frame(), &mut self.active_table);

        use consts::{KERNEL_HEAP_PML4, KERNEL_PML4};
        let e510 = self.active_table.p4()[KERNEL_PML4].clone();
        let e509 = self.active_table.p4()[KERNEL_HEAP_PML4].clone();

        self.active_table.with(&mut page_table, |pt: &mut Mapper| {
            set.map(pt);

            pt.p4_mut()[KERNEL_PML4] = e510;
            pt.p4_mut()[KERNEL_HEAP_PML4] = e509;
            pt.identity_map(Frame::of_addr(0xfee00000), EntryFlags::WRITABLE); // LAPIC
        });
        page_table
    }
    pub fn with(&mut self, page_table: InactivePageTable, mut f: impl FnMut()) -> InactivePageTable {
        let backup = self.active_table.switch(page_table);
        f();
        self.active_table.switch(backup)
    }
}
