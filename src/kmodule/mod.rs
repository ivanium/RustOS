use simple_filesystem::*;
use arch::driver::ide;
use core::{str, slice};
use alloc::boxed::Box;
use xmas_elf::{ElfFile, header::HeaderPt2, symbol_table::Binding, sections, sections::SectionHeader, sections::ShType};
use self::mod_loader::*;
use self::utils::*;
use self::consts::*;
use self::manager::*;
use alloc::string::String;
use spin::Mutex;

mod consts;
mod utils;
mod mod_loader;
pub mod manager;
#[macro_use]
mod export_func;

pub static ko_pool: Mutex<[u8; 1<<10]> = Mutex::new([0u8; 1<<10]);
pub static ko_pool_pointer: Mutex<u64> = Mutex::new(0);

pub static bss_pool: Mutex<[u64; 1<<10]> = Mutex::new([0; 1<<10]);
pub static bss_pool_ptr: Mutex<u64> = Mutex::new(0);

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


pub fn do_init_module(name: *const u8) -> i32 {
    let sfs = SimpleFileSystem::open(Box::new(&ide::DISK0)).unwrap();
    let root = sfs.root_inode();
    let files = root.borrow().list().unwrap();

    let slice = unsafe {
        let len = c_strlen(name);
        slice::from_raw_parts(name, len)
    };
    println!("\nin init module, mod name: {}", str::from_utf8(slice).unwrap());

    for fname in files.iter().filter(|&f| f == str::from_utf8(slice).expect("not a valid KM name"))  { // read file
        static mut BUF: [u8; 64 << 12] = [0; 64 << 12];
        let file = root.borrow().lookup(fname.as_str()).unwrap();
        let len = file.borrow().read_at(0, unsafe { &mut BUF }).unwrap();
        
        let mut info: elf_mod_info_s = 
                    elf_mod_info_s{ image: 0, image_size: 0, ptr:0, common_ptr: 0, common_size: 0, load_ptr: 0, unload_ptr: 0 };

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
        let export_symbol = true;

        unsafe {
            if elf_module_parse(&mut elf, &mut BUF, "", export_symbol, &mut info) != 0 {
                return -1;
            }
            add_module(&String::from_utf8_unchecked(slice.to_vec()), &info); // add module last since all of modifications of info are done before
        }

        println!("info.load_ptr: {:x}", info.load_ptr);
        // for i in 0x0..0x14 {
        //     unsafe { print!("{:x} ", *((info.load_ptr + i) as *const u8)); }
        // }
        // println!("");

        // call enter function
        let enter_func = &info.load_ptr as *const u64 as *const fn();
        unsafe {
            (*enter_func)();
        }
    } 
    0
}

pub fn do_cleanup_module(name: *const u8) -> i32 {
    use self::manager::*;
    use alloc::string::String;
    use core::slice;

    let slice = unsafe {
        let len = c_strlen(name);
        slice::from_raw_parts(name, len)
    };
    println!("in cleanup mod: {}, before clean up", str::from_utf8(slice).unwrap());
    print_modules();

    unsafe {
        if !module_loaded(&String::from_utf8_unchecked(slice.to_vec())) {
            println!("[ EE ] module {} not loaded.", String::from_utf8_unchecked(slice.to_vec()));
            return -1;
        }
    }
    unsafe { unload_module(&String::from_utf8_unchecked(slice.to_vec())) };
    println!("in cleanup mod: {}, after clean up", str::from_utf8(slice).unwrap());
    print_modules(); 
    0
}

pub fn print_modules() {
    print_loaded_module();
}

pub fn mod_init() {
    use self::export_func::*;
    
    println!("[ II ] mod init\n");
    let pos = ko_pool.lock().as_ptr() as u64;
    *ko_pool_pointer.lock() = pos;
    let pos = bss_pool.lock().as_ptr() as u64;
    *bss_pool_ptr.lock() = pos;

    mod_loader_init();

    export!("register_mod_add", register_mod_add);
    export!("unregister_mod_add", unregister_mod_add);
    export!("register_mod_mul", register_mod_mul);
    export!("unregister_mod_mul", unregister_mod_mul);
    export!("kprintf", kprintf);

    println!("[ II ] mod init finished\n");
}

pub unsafe fn do_mod_add(a: i32, b: i32) -> i32 {
    let mut c: i32 = 0;
    let idx = find_export_sym(MOD_ADD, false);
    if idx < 0 || get_sym_ptr(idx) == (-1) as i64 as u64 {
        println!("[ EE ] module add not loaded into kernel");
        return 0;
    }
    // println!("add fun ptr: {:#x}", get_sym_ptr(idx));
    // println!("c: {:#x}", &c as *const i32 as usize);
    // let fun_ptr = &(get_sym_ptr(idx)) as *const u64 as *const fn(i32, i32, *mut i32);
    let tmp: u64 = 0xffffff00001d04c8;
    let fun_ptr = &tmp as *const u64 as *const fn(i32, i32, *mut i32);

    // println!("fun_ptr= {:#x} ", (*fun_ptr) as usize);
    // let sptr1=0xffffff00001d0547 as usize;
    // println!("sptr1= {:#x} ", sptr1);
    // println!("*sptr1= {:x} ", *(sptr1 as *const u8));
    // let sptr2=0xffffff00001d0548 as usize;
    // println!("sptr2= {:#x} ", sptr2);
    // println!("*sptr2= {:x} ", *(sptr2 as *const u8));
    // for i in 0..50 {
    //     print!("{:x} ", *((sptr1 as usize + i) as *const u8))
    // }

    // println!("");
    // println!("test print_modules:{:#x}",&(print_modules as *const ()) as *const _ as *const u8 as u64);
    // println!("test mod_init:{:#x}",&(mod_init as *const ()) as *const _ as *const u8 as u64);
    println!("before add: a = {}, b = {}, c = {}", a, b, c);
    (*fun_ptr)(a, b, &mut c as *mut i32);
    println!("after add: a = {}, b = {}, c = {}", a, b, c);
    c
}

pub unsafe fn do_mod_mul(a: i32, b: i32) -> i32 {
    let mut c: i32 = 0;
    let idx = find_export_sym(MOD_MUL, false);
    if idx < 0 || get_sym_ptr(idx) == (-1) as i64 as u64 {
        println!("[ EE ] module add not loaded into kernel");
        return 0;
    }
    let fun_ptr = &(get_sym_ptr(idx)) as *const u64 as *const fn(i32, i32, *mut i32);
    (*fun_ptr)(a, b, &mut c as *mut i32);
    c
}
