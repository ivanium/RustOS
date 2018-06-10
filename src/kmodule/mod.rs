use simple_filesystem::*;
use arch::driver::ide;
use core::{str, slice};
use alloc::boxed::Box;
use xmas_elf::{ElfFile, header::HeaderPt2, symbol_table::Binding, sections, sections::SectionHeader, sections::ShType};
use self::mod_loader::*;

mod consts;
mod utils;
mod mod_loader;
pub mod manager;

#[derive(Clone)]
pub struct elf_mod_info_s {
    image : u64,
    image_size : usize,

    ptr : u64,
    common_ptr : u64,
    common_size : usize,
    load_ptr : u64,
    unload_ptr : u64,
}

pub fn do_init_module(name: *const u8, len: usize) -> i32 {
    let sfs = SimpleFileSystem::open(Box::new(&ide::DISK0)).unwrap();
    let root = sfs.root_inode();
    let files = root.borrow().list().unwrap();

    let slice = unsafe {slice::from_raw_parts(name, len) };

    for fname in files.iter().filter(|&f| f == str::from_utf8(slice).expect("not a valid KM name"))  { // read file
        static mut BUF: [u8; 64 << 12] = [0; 64 << 12];
        let file = root.borrow().lookup(fname.as_str()).unwrap();
        let len = file.borrow().read_at(0, unsafe { &mut BUF }).unwrap();
        
        add_module(name);

        let mut elf = ElfFile::new(unsafe{ &BUF[..len] }).expect("failed to read elf"); // get elf
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

        if elf_hdr_check(&elf) != 0 {
            return -1;
        }
        let export_symbol = false;
        let mut info: elf_mod_info_s = elf_mod_info_s{ image: 0, image_size: 0, ptr:0, common_ptr: 0, common_size: 0, load_ptr: 0, unload_ptr: 0 };
        unsafe {
            if elf_module_parse(&mut elf, &mut BUF, "", export_symbol, &mut info) != 0 {
                return -1;
            }
        }

        // call enter function
        let enter_func = info.load_ptr as *const fn();
        unsafe {
            (*enter_func)();
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

pub fn add_module(name: *const u8) -> i32 { // need a hash set
    0
}
