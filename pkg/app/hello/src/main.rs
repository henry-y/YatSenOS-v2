#![no_std]
#![no_main]

use core::alloc::Layout;

use lib::*;

extern crate lib;

struct Data {
    a: i32,
    b: i32,
}

fn main() -> isize {
    //println!("Hello, world!!!");
    stdout().write("Hello, world!!!\n");

    let layout = Layout::new::<Data>();

    let ptr = sys_allocate(&layout);

    println!("ptr: {:?}", ptr);

    let data = unsafe { &mut *(ptr as *mut Data) };



    // loop {}
    233
}

entry!(main);
