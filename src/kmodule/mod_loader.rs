use xmas_elf::{ElfFile, header::{HeaderPt1, HeaderPt2}, sections, symbol_table::Binding, sections::ShType};
use super::consts;
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

    println!("elf header check passed!");
    0
}

pub fn elf_module_parse<'a>(elf: &'a ElfFile<'a>, BUF: & mut [u8], export_symbol: bool) -> i32 {
    println!("sh count = {}, sh size = {}\n", elf.header.pt2.sh_count(), elf.header.pt2.sh_entry_size());

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
