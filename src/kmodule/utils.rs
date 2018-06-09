use xmas_elf::{ElfFile, sections::{SectionHeader, ShType}};

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


fn get_section_offset(sh: &SectionHeader, info: u32) -> u64 {
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

fn find_export_sym(name: &str, touch: bool) -> i32 {
    0
}

fn mod_touch_symbol(name: &str, ptr: *const u8, flags: u32) {

}

fn mod_disable_symbol(name: &str) {
    let idx = find_export_sym(name, true);
    if idx >= 0 {
        //
    }
}

fn elf_mod_create_symbol(name: &str, ptr: *const u8, flags: &mut u32) -> i32 {
    0
}

fn  elf_mod_get_symbol(name: &str, ptr: &mut *const u8, flags: &mut u32) -> i32 {
    0
}

fn fill_symbol_struct<'a>(elf: &'a ElfFile<'a>, export_symbol: i32) {

}

fn touch_export_sym(name: &str, ptr: u64, flags: u32) {

}
