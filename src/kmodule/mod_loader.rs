use xmas_elf::{ElfFile, header::{HeaderPt1, HeaderPt2, sanity_check}, sections, symbol_table::Binding, sections::ShType};
use super::consts;
use super::elf_mod_info_s;
use super::utils::*;

pub fn mod_loader_init() {
    // nothing to do here since all of the global variable are initialized when define
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

    match sanity_check(elf) {
        Ok(_)   => println!("elf header check passed!"),
        Err(_)  => {
            return -1;
        }
    }
    0
}

pub fn elf_module_parse<'a>(elf: &'a ElfFile<'a>, BUF: &mut [u8], name: &str, export_symbol: bool, info: &mut elf_mod_info_s) -> i32 {
    println!("sh count = {}, sh size = {}", elf.header.pt2.sh_count(), elf.header.pt2.sh_entry_size());
    info.image = BUF.as_ptr() as u64;
    info.image_size = BUF.len();

    let mut cur_common_alloc = 0;
    let mut cur_common_align = 1;

    let mut idx = 0;
    for sh in elf.section_iter() {
        use super::consts::*;
        use xmas_elf::sections::ShType;

        println!("sh name: {:?}, type: {:?}", sh.get_name(&elf), sh.get_type());
        sections::sanity_check(sh, &elf).unwrap();
        match sh.get_type() {
            Ok(ShType::SymTab) => {
                if let sections::SectionData::SymbolTable64(symtab) = sh.get_data(elf).ok().expect("not symtab 64") {
                    // println!("symtab:\n{:?}", symtab);
                    use xmas_elf::symbol_table::Entry;

                    let mut sym_idx = 0;
                    for sym in symtab {
                        if sym.shndx() != SHN_UNDEF && sym.shndx() < 0xff00 {
                        //    let sym_name = elf.get_string(sym.name()).ok().expect("get sym name error"); // TODO: uncertain
                            let sym_name = get_symbol_string(elf, BUF, sym.name());
                            match sym.get_binding() {
                                Ok(Binding::Local)  => {
                                    if sym_name == MOD_INIT_MODULE {
                                        //get section offset
                                        info.load_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();

                                    } else if sym_name == MOD_CLEANUP_MODULE {
                                        // get section offset
                                        info.unload_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();
                                    }

                                },
                                Ok(Binding::Global) => {
                                    println!("global symbol {}", sym_name);
                                    if sym_name == MOD_INIT_MODULE {
                                        // get section offset
                                        // println!("BUF:{:x}, sym.shndx: {}, sym.value: {}", BUF.as_ptr() as u64, sym.shndx(), sym.value());
                                        // print!("BUF init: ");
                                        // for i in 0x70..0x84 {
                                        //     print!("{:x} ", BUF[i]);
                                        // }
                                        // println!("");
                                        info.load_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();

                                    } else if sym_name == MOD_CLEANUP_MODULE {
                                        // get section offset
                                        info.unload_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();
                                    }
                                    if export_symbol {
                                        unsafe {
                                            // mod touch symbol
                                            println!("get sym: {}, offset: {}", sym_name, get_section_offset(elf, sym.shndx() as u32) + sym.value());
                                            // let tmp = (get_section_offset(elf, sym.shndx() as u32) + sym.value());
                                            // for i in tmp..tmp+0x40 {
                                            //     print!("{:x} ", BUF[i as usize]);
                                            // }
                                            // println!("");
                                            println!("{} addr {:#x}", sym_name, BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value());
                                            mod_touch_symbol(sym_name, BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value(), 0);
                                        }
                                    }
                                },
                                Ok(Binding::Weak)   => {
                                    if export_symbol {
                                        unsafe {
                                            // mod create symbol
                                            elf_mod_create_symbol(sym_name, (BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value()) as *mut u8, 0);
                                        }
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
                            unsafe {
                                // sym.set_value(cur_common_alloc); //TODO: fix modify memory containt
                                use xmas_elf::symbol_table::{Entry, Entry64};
                                let sym_ptr = (BUF.as_ptr() as u64 + sh.offset() + sym_idx) as *mut Entry64;
                                (*sym_ptr).set_value(cur_common_alloc);
                            }

                            cur_common_alloc += sym.size();
                        } else {
                            println!("shndx[{:#x}]", sym.shndx());
                        }
                        sym_idx += 1;
                    }
                }
            },
            Ok(ShType::NoBits) => {
                println!("bss section, alloc {} byte  align {:x}", sh.size(), sh.align());

                if bsf(sh.align()) != bsr(sh.align()) {
                    println!("bad align");
                    return -1;
                }

                if sh.align() > cur_common_align {
                    cur_common_align = sh.align();
                }
                if cur_common_alloc != 0 {
                    cur_common_alloc = ((cur_common_alloc - 1) | (sh.align() - 1)) + 1;
                }

                // sh.address() = cur_common_alloc; // TODO: fix modify address in hdr
                unsafe {
                    use xmas_elf::header::Class;
                    if elf.header.pt1.class() == Class::ThirtyTwo {
                        let sh_ptr = BUF.as_mut_ptr().offset(elf.header.pt2.sh_offset() as isize + idx*elf.header.pt2.sh_entry_size() as isize) as *mut u32;
                        *sh_ptr.offset(3) = cur_common_alloc as u32; // hard code
                    } else if elf.header.pt1.class() == Class::SixtyFour {
                        let sh_ptr = BUF.as_mut_ptr().offset(elf.header.pt2.sh_offset() as isize + idx*elf.header.pt2.sh_entry_size() as isize) as *mut u64;
                        *sh_ptr.offset(4) = cur_common_alloc;
                    }
                }
                cur_common_alloc += sh.size();
            },
            _       => {},
        }
        idx += 1;
    }

    let mut common_space: u64 = 0;
    if cur_common_align > consts::PGSIZE {
        debug!("align failed\n");
        return -1;
    } else if cur_common_alloc > 0 {
        // kmalloc
        // common_space = xxx
        // if
        // println!("no enough memory for bss\n");
        use super::bss_pool_ptr;
        common_space = *bss_pool_ptr.lock();
        if common_space == 0 {
            println!("no enough memory for bss");
            return -1;
        }
        *bss_pool_ptr.lock() += cur_common_alloc;

        println!("memory pointer: {:16x}", common_space);
        unsafe {
            c_memset(common_space as *mut u8, 0, cur_common_alloc as usize);
        }

        info.common_ptr = common_space;
        info.common_size = cur_common_alloc as usize;
        // return -1;
    } else {
        info.common_ptr = 0;
        info.common_size = 0;
    }

    // fill the relocation entries
    for sh in elf.section_iter() {
        match sh.get_type() {
            Ok(ShType::Rela) => {
                if let sections::SectionData::Rela64(reloc_list) = sh.get_data(elf).ok().expect("get rela table failed") {
                    for reloc in reloc_list {
                        //symtab = fill symbol struct
                        let mut symtab = fill_symbol_struct(elf, BUF, reloc.get_symbol_table_index());

                        let mem_addr = BUF.as_ptr() as u64 + get_section_offset(elf, sh.info()) + reloc.get_offset();
                        let mut reloc_addr: u64 = 0;

                        unsafe {
                            use super::consts::*;
                            use xmas_elf::symbol_table::Entry;

                            // println!("reloc[{:2x}] offset[{:#08x}] for [{}], sym offset[{:#08x}]", reloc.get_type(), reloc.get_offset(), elf.get_string((*symtab).name()).unwrap(), (*symtab).value());
                            if (*symtab).shndx() == SHN_UNDEF {
                                let sym_name = elf.get_string((*symtab).name()).ok().expect("get sym name error");
                                let idx = find_export_sym(sym_name, false);
                                if idx == -1 {
                                    if sym_name == name {
                                        reloc_addr = BUF.as_ptr() as u64;
                                    } else {
                                        println!("Error: unresolved symbol: {}", sym_name);
                                        reloc_addr = 0;
                                    }
                                }  else {
                                    println!("external symbol {} addr = {:#0x}", sym_name, ex_sym_ptr.lock()[idx as usize]);
                                    reloc_addr = ex_sym_ptr.lock()[idx as usize];
                                }
                            } else if (*symtab).shndx() < 0xff00 {
                                // println!("section offset {:#016x}, addr {:#016x}", get_section_offset(elf, (*symtab).shndx() as u32), (*symtab).value());
                                let sym_sh = elf.section_header((*symtab).shndx()).unwrap();
                                if sym_sh.get_type().unwrap() == ShType::NoBits {
                                    reloc_addr = common_space;
                                } else {
                                    reloc_addr = BUF.as_ptr() as u64;
                                }
                                reloc_addr += get_section_offset(elf, (*symtab).shndx() as u32);
                                reloc_addr += (*symtab).value();
                            } else if (*symtab).shndx() == SHN_COMMON {
                                reloc_addr = common_space + (*symtab).value();
                            } else {
                                debug!("Error: unhandled sym shndx");
                            }
                        }
                        // println!("reloc_addr: {:#x}, reloc.addend: {:#x}", reloc_addr, reloc.get_addend());
                        let mut val = (reloc_addr as i64 + reloc.get_addend() as i64) as u64;

                        use super::consts::*;
                        unsafe {
                            match reloc.get_type() {
                                R_X86_64_NONE   => {},
                                R_X86_64_64     => {
                                    *(mem_addr as *mut u64) = val;
                                },
                                R_X86_64_32     => {
                                    *(mem_addr as *mut u32) = val as u32; 
                                },
                                R_X86_64_32S    => {
                                    *(mem_addr as *mut i32) = val as i32;
                                    println!("32s val: {:#x}, mem_addr: {:#x}, *mem_addr: {:#x}", val, mem_addr, *(mem_addr as *const i32));
                                },
                                R_X86_64_PC32   => {
                                    println!("pc32 val: {:#x}, mem_addr: {:#x}", val, mem_addr);
                                    let val32 = (val as i64 - mem_addr as i64) as u32;
                                    *(mem_addr as *mut u32) = val32;
                                },
                                R_X86_64_PLT32  => {
                                    println!("plt32 val: {:#x}, mem_addr: {:#x}", val, mem_addr);
                                    let val32 = (val as i64 - mem_addr as i64) as u32;
                                    *(mem_addr as *mut u32) = val32; 
                                },
                                _               => {
                                    println!("unsupported relocation type ({:#x})", reloc.get_type());
                                },
                            }
                        }
                    }
                }
            },
            Ok(ShType::Rel)  => {
                debug!("Error: relocation TYPE rel not implmented");
            },
            Ok(_)            => {},
            Err(_)           => println!("Error in get sh type"),
        }
    }
    // for ph in elf.program_iter() {
    //     println!("ph: {:?}", ph);
    // }
    0
}

pub fn unload_module(name: &str) -> i32 {
    use super::manager::*;
    use alloc::string::String;

    let info = get_module(&String::from(name));
    match info {
        Some(mod_info) => {
            unsafe { 
                let exit_func = &mod_info.unload_ptr as *const u64 as *const fn();
                (*exit_func)();
            }
            return del_module(&String::from(name))
        }
        None           => {
            println!("module info not found for {}", name);
            -1;
        }
    }
    0
}