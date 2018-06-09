use simple_filesystem::*;
use arch::driver::ide;
use core::{str, slice};
use alloc::boxed::Box;
use xmas_elf::{ElfFile, header::HeaderPt2};
use xmas_elf::symbol_table::Binding;
use xmas_elf::sections;

mod consts;
pub mod manager;

#[derive(Clone)]
pub struct elf_mod_info_s {
    image : u64,
    image_size : u32,
     
    ptr : u64,
    common_ptr : u64,
    common_size : u32,
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
        unsafe {
            if elf_module_parse(&mut elf, &mut BUF, export_symbol) != 0 {
                return -1;
            }
        }

        // call enter function
    } 
    0
}

pub fn do_cleanup_module(name: *const char) -> i32 {
    0
}

pub fn print_modules() {
    println!("in print_modules");
}

pub fn elf_hdr_check<'a>(elf: &'a ElfFile<'a>) -> i32 {
    debug!("begin elf header check!");
    if elf.header.pt1.magic != [0x7F, 0x45, 0x4c, 0x46] {
        println!("elf hdr magic {:?}", elf.header.pt1.magic);
        return -1;
    }

    match elf.header.pt1.class() {
        ThirtyTwo => println!("machine type: x86 32"),
        SixtyFour => println!("machine type: x86 64"),
        _         => {
            println!("machine type: not recognized");
            return -1;
        },
    }

    match elf.header.pt2.type_().as_type() {
        Relocatable => println!("elf type: relocatable"),
        _           => {
            println!("elf type error!");
            return -1;
        },
    }

    match elf.header.pt2 {
        HeaderPt2::Header32(_) => println!("error ELF type: 32"),
        HeaderPt2::Header64(_) => println!("ELF type: 64"),
    }

    match elf.header.pt2.machine() {
        X86     => println!("machine type: x86 32"),
        X86_64  => println!("machine type: x86 64"),
        _       => {
            println!("machine type: not support");
            return -1;
        },
    }

    if elf.header.pt2.entry_point() != 0 {
        println!("error entry point {}", elf.header.pt2.entry_point());
        return -1;
    }

    println!("elf header check passed!");
    0
}

pub fn elf_module_parse<'a>(elf: &'a mut ElfFile<'a>, BUF: & mut [u8], export_symbol: bool) -> i32 {
    println!("sh count = {}, sh size = {}\n", elf.header.pt2.sh_count(), elf.header.pt2.sh_entry_size());

    let mut cur_common_alloc = 0;
    let mut cur_common_align = 1;

    for sh in elf.section_iter() {
        use self::consts::*;

        println!("sh name: {:?}, type: {:?}", sh.get_name(&elf), sh.get_type());
        sections::sanity_check(sh, &elf).unwrap();
        match sh.get_type() {
            SymTab => {
                if let sections::SectionData::SymbolTable64(symtab) = sh.get_data(elf).ok().expect("not symtab 64") {
                    println!("symtab: {:?}", symtab);
                    use xmas_elf::symbol_table::Entry;
                    for sym in symtab {
                        if sym.shndx() != SHN_UNDEF && sym.shndx() < 0xff00 {
                            let sh_name = sh.get_name(elf).ok().expect("get sh name error!");
                            match sym.get_binding() {
                                Ok(Binding::Local)  => {
                                    if sh_name == MOD_INIT_MODULE {
                                        //get section offset

                                    } else if sh_name == MOD_CLEANUP_MODULE {
                                        // get section offset
                                    }

                                },
                                Ok(Binding::Global) => {
                                    println!("global symbal");
                                    if sh_name == MOD_INIT_MODULE {
                                        // get section offset
                                    } else if sh_name == MOD_CLEANUP_MODULE {
                                        // get section offset
                                    }
                                    if export_symbol {
                                        // mod touch symbol
                                    }
                                },
                                Ok(Binding::Weak)   => {
                                    if export_symbol {
                                        // mod create symbol
                                    }
                                },
                                Ok(_)               => {
                                    println!("unrecognized sym binding");
                                }
                                Err(_)              => {
                                    println!("error sym binding!");
                                }
                            };
                        } else if sym.shndx() == SHN_COMMON {
                            println!("SHN_COMMON, alloc {} byte offset {}\n", sym.size(), cur_common_alloc);
                            // sym.set_value(cur_common_alloc); //TODO: fix modify memory containt
                            cur_common_alloc += sym.size();
                        } else {
                            println!("shndx[{}]\n", sym.shndx());
                        }
                    }
                }
            },
            NoBits => {
                println!("bss section, alloc {} byte  align {:x}", sh.size(), sh.align());

                if bsf(sh.align()) != bsr(sh.align()) {
                    println!("bad align");
                    return -1;
                }

                if sh.align() > cur_common_align {
                    cur_common_align = sh.align();
                }
                cur_common_alloc = ((cur_common_alloc - 1) | (sh.align() - 1)) + 1;
                // sh.address() = cur_common_alloc; // TODO: fix modify address in hdr
                cur_common_alloc += sh.size();
            },
        }
    }

    let common_space: u64;
    if cur_common_align > consts::PGSIZE {
        debug!("align failed\n");
        return -1;
    } else if cur_common_alloc > 0 {
        // kmalloc
        // if
        // println!("no enough memory for bss\n");
        // set common data
        // set common size
        // return -1;
    } else {
        // set common data
        // set common size
    }

    // fill the relocation entry
    for sh in elf.section_iter() {
        use xmas_elf::sections::ShType;
        match sh.get_type() {
            Ok(ShType::Rela) => {
                //
            },
            Ok(ShType::Rel)  => {
                debug!("Error: relocation TYPE rel not implmented");
            },
            Ok(_)            => {},
            Err(_)           => println!("Error in get sh type"),
        }
    }
    for ph in elf.program_iter() {
        println!("ph: {:?}", ph);
    }
    0
}

pub fn add_module(name: *const u8) -> i32 { // need a hash set
    0
}

pub fn bsf(n: u64) -> i32 {
    if n == 0 {
        return -1;
    }
    let mut res: u64;
    unsafe {
        asm!("bsfq $1, $0"
            : "=r" (res)
            : "r" (n)
            : "volatile");
    }
    return res as i32;
}

pub fn bsr(n: u64) -> i32 {
	if n == 0 {
        return -1;
    }
	let mut res: u64;
	unsafe {
        asm!("bsrq %1, %0"
            : "=r" (res)
            : "r" (n)
            : "volatile");
    }
	return res as i32;
}

#[repr(C, packed)]
pub struct SymTab_s {
    sym_name: u32,
    sym_info: u8,
    sym_other: u8,
    sym_shndx: u16,
    sym_address: u64,
    sym_size: u64,
}