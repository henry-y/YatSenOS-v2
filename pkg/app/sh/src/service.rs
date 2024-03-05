use lib::*;
use alloc::{format, string::*, vec};


pub fn cat(path: &str, root_dir: &mut String) {
    lib::println!("havn't impl this function");
    // if(path.starts_with('/')) {

    // }
}

pub fn ls(root_dir: &String) {
    sys_list_dir(root_dir.as_str());
    lib::println!("havn't impl this function");
}

pub fn list_app() {
    sys_list_app();
}

pub fn list_proc() {
    sys_stat();
}

pub fn cd(dir: &str, root_dir: &mut String) {
    lib::println!("cd");
}

pub fn exec(file: &str) -> u16 {
    let ret = sys_spawn(file);
    return ret;
}