use xmas_elf::{ElfFile, sections::{SectionHeader, ShType}, symbol_table::Entry64};
use super::consts::{EXPORT_SYM_HASH, EXPORT_SYM_COUNT_MAX, EXPORT_SYM_NAME_LEN};
use spin::Mutex;
use core::{slice, str};

pub static mut ex_sym_f: Mutex<[i32; EXPORT_SYM_HASH]> = Mutex::new([-1; EXPORT_SYM_HASH]);
pub static mut ex_sym_name: Mutex<[[u8; EXPORT_SYM_NAME_LEN]; EXPORT_SYM_COUNT_MAX]>
                                = Mutex::new([[0;EXPORT_SYM_NAME_LEN]; EXPORT_SYM_COUNT_MAX]);
pub static mut ex_sym_ptr: Mutex<[u64; EXPORT_SYM_COUNT_MAX]> = Mutex::new([0; EXPORT_SYM_COUNT_MAX]);
pub static mut ex_sym_flags: Mutex<[u32; EXPORT_SYM_COUNT_MAX]> = Mutex::new([0; EXPORT_SYM_COUNT_MAX]);
pub static mut ex_sym_n: Mutex<[i32; EXPORT_SYM_COUNT_MAX]> = Mutex::new([-1; EXPORT_SYM_COUNT_MAX]);

pub static mut ex_sym_count: Mutex<i32> = Mutex::new(0);

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
        },
        Err(err)           => {
            println!("{}", err);
            0
        }
    }
}

pub unsafe fn find_export_sym(name: &str, touch: bool) -> i32 {
    let name_len = name.len();
    let h = sym_hash(name, name_len);
    let mut cur = ex_sym_f.lock()[h];

    while cur != -1 {
        let sym_name = ex_sym_name.lock()[cur as usize];
        if str::from_utf8_unchecked(&sym_name) == name {
            break;
        } else {
            cur = ex_sym_n.lock()[cur as usize];
        }
    }

    if cur == -1 && touch {
        cur = *ex_sym_count.lock();
        *ex_sym_count.lock() += 1;
        ex_sym_n.lock()[cur as usize] = ex_sym_f.lock()[h];
        ex_sym_f.lock()[h] = cur;

        ex_sym_name.lock()[cur as usize].clone_from_slice(&name.as_bytes()[0..50]);
    }

    cur
}

pub unsafe fn mod_touch_symbol(name: &str, ptr: u64, flags: u32) {
    let idx = find_export_sym(name, true);
    ex_sym_ptr.lock()[idx as usize] = ptr;
    ex_sym_flags.lock()[idx as usize] = flags;
}

pub unsafe fn mod_disable_symbol(name: &str) {
    let idx = find_export_sym(name, true);
    if idx >= 0 {
        ex_sym_ptr.lock()[idx as usize] = 0;
    }
}

pub unsafe fn elf_mod_create_symbol(name: &str, ptr: *const u8, flags: u32) -> i32 {
    let idx = find_export_sym(name, true);
    if idx != *ex_sym_count.lock() - 1 {
        return -1;
    }
    ex_sym_ptr.lock()[idx as usize] = ptr as u64;
    ex_sym_flags.lock()[idx as usize] = flags;
    0
}

pub unsafe fn elf_mod_get_symbol(name: &str, ptr: &mut *const u8, flags: &mut u32) -> i32 {
    let idx = find_export_sym(name, false);
    if idx == -1 {
        return -1;
    }
    *ptr = ex_sym_ptr.lock()[idx as usize] as *const u8;
    *flags = ex_sym_flags.lock()[idx as usize];
    0
}

pub fn fill_symbol_struct<'a>(elf: &'a ElfFile<'a>, BUF: &mut [u8], symbol: u32) -> *mut Entry64 {
    for sh in elf.section_iter().filter(|&sh| sh.get_type().unwrap() == ShType::SymTab) {
        let symtab = (BUF.as_ptr() as u64 + sh.offset() as u64 + (symbol as u64 * sh.entry_size())) as *mut Entry64;
        return symtab;
    }
    return 0 as *mut Entry64;
}

pub unsafe fn touch_export_sym(name: &str, ptr: u64, flags: u32) {
    let name_len = name.len();
    let h = sym_hash(name, name_len);
    let mut cur = ex_sym_f.lock()[h];

    while cur != -1 {
        if str::from_utf8_unchecked(&ex_sym_name.lock()[cur as usize]) == name {
            break;
        } else {
            cur = ex_sym_n.lock()[cur as usize];
        }
    }

    if cur == -1 {
        cur = *ex_sym_count.lock() as i32;
        *ex_sym_count.lock() += 1;
        ex_sym_n.lock()[cur as usize] = ex_sym_f.lock()[h];
        ex_sym_f.lock()[h] = cur;

        ex_sym_name.lock()[cur as usize].clone_from_slice(&name.as_bytes()[0..50]);
    }

    ex_sym_ptr.lock()[cur as usize] = ptr;
    ex_sym_flags.lock()[cur as usize] = flags;
}

pub fn sym_hash(name: &str, len: usize) -> usize {
    let mut ret: usize = 0;
    for c in name.as_bytes() {
        ret = (ret*13 + *c as usize) % EXPORT_SYM_HASH;
    }
    return ret;
}