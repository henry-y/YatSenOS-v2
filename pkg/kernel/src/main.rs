#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;
use log::*;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    trace!("[+] begin list_app...");
    crate::proc::list_app();
    trace!("[+] list_app ok...");
    ysos::wait(spawn_init());
    ysos::proc::print_process_list();
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    println!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    // proc::spawn("hello").unwrap();
    proc::spawn("test").unwrap()
}
