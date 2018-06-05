use simple_filesystem::*;
use arch::driver::ide;
use core::{str, slice};
use alloc::boxed::Box;
use xmas_elf::{ElfFile, header::HeaderPt2};

pub fn do_init_module(name: *const u8, len: usize) -> i32 {
    let sfs = SimpleFileSystem::open(Box::new(&ide::DISK0)).unwrap();
    let root = sfs.root_inode();
    let files = root.borrow().list().unwrap();

    let slice = unsafe {slice::from_raw_parts(name, len) };

    for fname in files.iter().filter(|&f| f == str::from_utf8(slice).expect("not a valid KM name"))  {
        static mut BUF: [u8; 64 << 12] = [0; 64 << 12];
        let file = root.borrow().lookup(fname.as_str()).unwrap();
        let len = file.borrow().read_at(0, unsafe { &mut BUF }).unwrap();
        let elf = ElfFile::new(unsafe{ &BUF[..len] }).expect("failed to read elf");
        let is32 = match elf.header.pt2 {
            HeaderPt2::Header32(_) => true,
            HeaderPt2::Header64(_) => false,
        };
        println!("elf hdr: {:?}", elf.header);
        for sh in elf.section_iter() {
            println!("sh: {:?}", sh);
        }
        for ph in elf.program_iter() {
            println!("ph: {:?}", ph);
        }
        if elf.header.pt1.magic != [0x7F, 0x45, 0x4c, 0x46] {
            println!("elf hdr magic {:?}", elf.header.pt1.magic);
        }
    } 
    0
}

pub fn do_cleanup_module(name: *const char) -> i32 {
    0
}

pub fn print_modules() {
    println!("in print_modules");
}