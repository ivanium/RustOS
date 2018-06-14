use simple_filesystem::*;
use alloc::boxed::Box;
use arch::driver::ide;
use spin::Mutex;
use process;
use memory::{MemoryController, ActivePageTable, MemoryArea, MemorySet, EntryFlags};
use process::MC;

pub fn load_sfs() {
//    let slice = unsafe { MemBuf::new(_binary_user_ucore32_img_start, _binary_user_ucore32_img_end) };
    let sfs = SimpleFileSystem::open(Box::new(&ide::DISK0)).unwrap();
    let root = sfs.root_inode();
    let files = root.borrow().list().unwrap();
    // trace!("Loading programs: {:?}", files);
    debug!("Loading programs: {:?}", files);

//    for name in files.iter().filter(|&f| f != "." && f != "..") {
    for name in files.iter().filter(|&f| f == "insmod") {
        static mut BUF: [u8; 64 << 12] = [0; 64 << 12];
        let file = root.borrow().lookup(name.as_str()).unwrap();
        let len = file.borrow().read_at(0, unsafe { &mut BUF }).unwrap();
        process::add_user_process(name, unsafe { &BUF[..len] });
    }

    // process::print();

    use xmas_elf::{ElfFile, header::HeaderPt2, program::{Flags, ProgramHeader}};

    for name in files.iter().filter(|&f| f == "rmmod") {
        static mut BUF: [u8; 64 << 12] = [0; 64 << 12];
        let file = root.borrow().lookup(name.as_str()).unwrap();
        let len = file.borrow().read_at(0, unsafe { &mut BUF }).unwrap();
        process::add_user_process(name, unsafe { &BUF[..len] });
        // let elf = ElfFile::new(unsafe{ &BUF[..len] }).expect("failed to read elf");
        // let is32 = match elf.header.pt2 {
        //     HeaderPt2::Header32(_) => true,
        //     HeaderPt2::Header64(_) => false,
        // };
        // println!("elf hdr:\n{:?}", elf.header);
        // for sh in elf.section_iter() {
        //     println!("sh: {:?}", sh);
        // }
        // for ph in elf.program_iter() {
        //     println!("ph: {:?}", ph);
        // }
        // if elf.header.pt1.magic != [0x7F, 0x45, 0x4c, 0x46] {
        //     println!("elf hdr magic {:?}", elf.header.pt1.magic);
        // }

        // let mut mc = MC.try().unwrap().lock();
        // let mut memory_set = MemorySet::from(&elf);
        // let page_table = mc.make_page_table(&memory_set);
        // trace!("{:#x?}", memory_set);
        // let page_table = mc.with(page_table, || {
        //     for ph in elf.program_iter() {
        //         let (virt_addr, offset, file_size) = match ph {
        //             ProgramHeader::Ph32(ph) => (ph.virtual_addr as usize, ph.offset as usize, ph.file_size as usize),
        //             ProgramHeader::Ph64(ph) => (ph.virtual_addr as usize, ph.offset as usize, ph.file_size as usize),
        //         };
        //         let target = unsafe { slice::from_raw_parts_mut(virt_addr as *mut u8, file_size) };
        //         target.copy_from_slice(unsafe { &BUF[offset..offset + file_size] });
        //     }
        // });

        // let mut active_table = unsafe { ActivePageTable::new() };
        // let old_table = active_table.switch(page_table);

        // let entry_fn: fn() -> i32;
        // unsafe {
        //     (*(elf.header.pt2.entry_point() as *mut (fn() ->i32)))();
        // }
    }
    process::print();
    use process::PROCESSOR;
    let mut processor = PROCESSOR.try().unwrap().lock();
    processor.set_reschedule();
}

#[cfg(feature = "link_user_program")]
extern {
    fn _binary_user_ucore32_img_start();
    fn _binary_user_ucore32_img_end();
    fn _binary_user_xv6_64_img_start();
    fn _binary_user_xv6_64_img_end();
}

struct MemBuf(&'static [u8]);

impl MemBuf {
    unsafe fn new(begin: unsafe extern fn(), end: unsafe extern fn()) -> Self {
        use core::slice;
        MemBuf(slice::from_raw_parts(begin as *const u8, end as usize - begin as usize))
    }
}

impl Device for MemBuf {
    fn read_at(&mut self, offset: usize, buf: &mut [u8]) -> Option<usize> {
        let slice = self.0;
        let len = buf.len().min(slice.len() - offset);
        buf[..len].copy_from_slice(&slice[offset..offset + len]);
        Some(len)
    }
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> Option<usize> {
        None
    }
}

use core::slice;

impl BlockedDevice for &'static ide::DISK0 {
    fn block_size_log2(&self) -> u8 {
        debug_assert_eq!(ide::BLOCK_SIZE, 512);
        9
    }
    fn read_at(&mut self, block_id: usize, buf: &mut [u8]) -> bool {
        assert!(buf.len() >= ide::BLOCK_SIZE);
        let buf = unsafe { slice::from_raw_parts_mut(buf.as_ptr() as *mut u32, ide::BLOCK_SIZE / 4) };
        self.0.lock().read(block_id as u64, 1, buf).is_ok()
    }
    fn write_at(&mut self, block_id: usize, buf: &[u8]) -> bool {
        assert!(buf.len() >= ide::BLOCK_SIZE);
        let buf = unsafe { slice::from_raw_parts(buf.as_ptr() as *mut u32, ide::BLOCK_SIZE / 4) };
        self.0.lock().write(block_id as u64, 1, buf).is_ok()
    }
}