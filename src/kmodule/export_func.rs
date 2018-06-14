use super::utils::*;
use super::consts::*;

macro_rules! export {
    ($name: expr, $func: ident) => {
        unsafe { touch_export_sym(&$name, $func as *const u8 as u64, 0); }
    };
}

#[no_mangle]
pub unsafe extern "C" fn register_mod_add(f: *const u8) {
    mod_touch_symbol(MOD_ADD, f as u64, 0);
}

#[no_mangle]
pub unsafe extern "C" fn unregister_mod_add() {
    mod_disable_symbol(MOD_ADD);
}

#[no_mangle]
pub unsafe extern "C" fn register_mod_mul(f: *const u8) {
    mod_touch_symbol(MOD_MUL, f as u64, 0);
}

#[no_mangle]
pub unsafe extern "C" fn unregister_mod_mul() {
    mod_disable_symbol(MOD_MUL);
}

#[no_mangle]
pub unsafe extern "C" fn kprintf(fmt: *const u8) -> i32 { // support only string now
    use core::{str, slice};
    use alloc::string::String;
    
    let mut len = 0;
    while *((fmt as usize + len) as *const u8) != 0 {
        len += 1;
    }

    print!("{}", String::from_utf8_unchecked(slice::from_raw_parts(fmt, len).to_vec()));
    0
}