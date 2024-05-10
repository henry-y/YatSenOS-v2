#![no_std]
#![no_main]

use core::alloc::Layout;

use lib::*;

extern crate lib;


fn main() -> isize {
    let ptr = 0xffffff00008efd68 as *mut u64;

    let data = unsafe { &mut *(ptr as *mut u64) };

    println!("data: {:?}", data);
    
    0
}

entry!(main);
