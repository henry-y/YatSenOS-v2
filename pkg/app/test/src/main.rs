#![no_std]
#![no_main]
const MOD: u64 = 1000000007;

use lib::*;

extern crate lib;

fn factorial(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1) % MOD
    }
}

fn main() -> isize {
    print!("Input n: ");

    let input = lib::stdin().read_line();

    println!("input succ...");

    // prase input as u64
    let n = input.parse::<u64>().unwrap();
    println!("n: {}", n);
    
    if n > 1000000 {
        println!("n must be less than 1000000");
        return 1;
    }

    // calculate factorial
    let result = factorial(n);

    println!("factorial succ...");
    
    //let result = 1;

    // print system status
    sys_stat();

    // print result
    println!("The factorial of {} under modulo {} is {}.", n, MOD, result);

    0
}

entry!(main);