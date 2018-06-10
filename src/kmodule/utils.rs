use xmas_elf::{ElfFile, sections::{SectionHeader, ShType}, symbol_table::Entry64};

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


pub fn get_section_offset<'a>(elf: &'a ElfFile<'a>, info: u32) -> u64 {
    let sh = elf.section_header(info as u16).ok().expect("get sh offset failed!");
    match sh.get_type() {
        Ok(ShType::NoBits) => {
            sh.address()
        },
        Ok(_)              => {
            sh.offset()
        }
        Err(err)           => {
            println!("{}", err);
            0
        }
    }
}

pub fn find_export_sym(name: &str, touch: bool) -> i32 {
    0
}

pub fn mod_touch_symbol(name: &str, ptr: u64, flags: u32) {

}

pub fn mod_disable_symbol(name: &str) {
    let idx = find_export_sym(name, true);
    if idx >= 0 {
        //
    }
}

pub fn elf_mod_create_symbol(name: &str, ptr: *const u8, flags: u32) -> i32 {
    0
}

pub fn elf_mod_get_symbol(name: &str, ptr: &mut *const u8, flags: &mut u32) -> i32 {
    0
}

pub fn fill_symbol_struct<'a>(elf: &'a ElfFile<'a>, BUF: &mut [u8], symbol: u32) -> *mut Entry64 {
    for sh in elf.section_iter().filter(|&sh| sh.get_type().unwrap() == ShType::SymTab) {
        let symtab = (BUF.as_ptr() as u64 + sh.offset() as u64 + (symbol as u64 * sh.entry_size())) as *mut Entry64;
        return symtab;
    }
    return 0 as *mut Entry64;
}

pub fn touch_export_sym(name: &str, ptr: u64, flags: u32) {

}
