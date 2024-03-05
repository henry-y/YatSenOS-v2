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

pub fn exec(path: &str)  {

    let pid = sys_spawn(path);

    if pid == 0 {
        errln!("failed to spawn process: {}", path);
        return;
    }

    let ret = sys_wait_pid(pid);

    println!(
        "[+] process exited with code {}",
        ret    
    );
}