use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use super::elf_mod_info_s;

lazy_static! {
	pub static ref modules: Mutex<Vec<(String,elf_mod_info_s)>> = Mutex::new(vec![]);  //模块信息
}

fn hash_find(name: &String)-> Option<usize>{
    let mut res:usize = 0;
    for item in modules.lock().iter() {
        if *name == item.0 {
            return Some(res);
        }
        res=res+1;
    }
    None
}

pub fn add_module(name: &String, info: &elf_mod_info_s) {  //将模块加入hash表
    modules.lock().push((name.clone(),info.clone()));
}

pub fn del_module(name: &String) -> i32 {  //将模块移除hash表
    match hash_find(name)
        {
            Some(idx) => { modules.lock().remove(idx); 0 },
            None => { -1 },
        }
}

pub fn get_module(name: &String)-> Option<elf_mod_info_s> {  //get名为name的模块信息
    match hash_find(name)
        {
            Some(idx) => {
                match modules.lock().get(idx)
                    {
                        Some(res) => Some(res.1.clone()),
                        None => None,
                    }
            },
            None => None,
        }
}

/**
 *  return whether a module has been loaded
 *  return 1 means loaded, 0 means not loaded
 */
pub fn module_loaded(name: &String)-> bool {
    match hash_find(name)
        {
            Some(_) => true,
            None => false,
        }
}

pub fn print_loaded_module() {
    info!("module list:");
    for key in modules.lock().iter() {
        info!("{:?}", key.0);
    }
}