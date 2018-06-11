use xmas_elf::{ElfFile, header::{HeaderPt1, HeaderPt2, sanity_check}, sections, symbol_table::Binding, sections::ShType};
use super::consts;
use super::elf_mod_info_s;
use super::utils::*;

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
    println!("sh count = {}, sh size = {}\n", elf.header.pt2.sh_count(), elf.header.pt2.sh_entry_size());
    info.image = BUF.as_ptr() as u64;
    info.image_size = BUF.len();

    let mut cur_common_alloc = 0;
    let mut cur_common_align = 1;

    for sh in elf.section_iter() {
        use super::consts::*;

        println!("sh name: {:?}, type: {:?}", sh.get_name(&elf), sh.get_type());
        sections::sanity_check(sh, &elf).unwrap();
        match sh.get_type() {
            SymTab => {
                if let sections::SectionData::SymbolTable64(symtab) = sh.get_data(elf).ok().expect("not symtab 64") {
                    println!("symtab: {:?}", symtab);
                    use xmas_elf::symbol_table::Entry;
                    for sym in symtab {
                        if sym.shndx() != SHN_UNDEF && sym.shndx() < 0xff00 {
                            let sym_name = elf.get_string(sym.name()).ok().expect("get sym name error"); // uncertain
//                            let sh_name = sh.get_name(elf).ok().expect("get sh name error!");
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
                                    println!("global symbal");
                                    if sym_name == MOD_INIT_MODULE {
                                        // get section offset
                                        info.load_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();

                                    } else if sym_name == MOD_CLEANUP_MODULE {
                                        // get section offset
                                        info.unload_ptr = BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value();
                                    }
                                    if export_symbol {
                                        unsafe {
                                            // mod touch symbol
                                            mod_touch_symbol(sym_name, BUF.as_ptr() as u64 + get_section_offset(elf, sym.shndx() as u32) + sym.value(), 0);
                                        }
                                    }
                                },
                                Ok(Binding::Weak)   => {
                                    if export_symbol {
                                        unsafe {
                                            // mod create symbol
                                            elf_mod_create_symbol(sym_name, (BUF.as_ptr() as u64 + sym.value()) as *mut u8, 0);
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

    let mut common_space: u64 = 0;
    if cur_common_align > consts::PGSIZE {
        debug!("align failed\n");
        return -1;
    } else if cur_common_alloc > 0 {
        // kmalloc
        // common_space = xxx
        // if
        // println!("no enough memory for bss\n");
        // set common data
        // set common size
        info.common_ptr = common_space;
        info.common_size = cur_common_alloc as usize;
        // return -1;
    } else {
        // set common data
        // set common size
        info.common_ptr = 0;
        info.common_size = 0;
    }

    // fill the relocation entry
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

                            println!("reloc[{:2x}] offset[{:8x}] for [{}], sym offset[{:8x}]\n", reloc.get_type(), reloc.get_offset(), elf.get_string((*symtab).name()).unwrap(), (*symtab).value());
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
                                    // println!("external symbol %s addr = %p\n", sym_name, ex_sym_ptr[idx]);
                                    // reloc_addr = ex_sym_ptr[idx];
                                }
                            } else if (*symtab).shndx() < 0xff00 {
                                println!("section offset {:16x}, addr {:16x}", get_section_offset(elf, (*symtab).shndx() as u32), (*symtab).value());
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
                        let mut val = reloc_addr + reloc.get_addend();

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
                                },
                                R_X86_64_PC32   => {
                                    val -= mem_addr as u64;
                                    *(mem_addr as *mut u64) = val;
                                },
                                _               => {
                                    println!("unsupported relocation type ({:x})\n", reloc.get_type());
                                },
                            }
                            println!("fill rel address {:8x} to {:8x}", *(mem_addr as *const u32), mem_addr);
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
