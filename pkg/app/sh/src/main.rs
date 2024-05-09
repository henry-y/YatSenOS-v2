// write a shell code
#![no_std]
#![no_main]

extern crate alloc;

mod service;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use lib::*;

extern crate lib;

use lib::io::stdin;
use lib::{print, println};

fn main() -> isize {
    println!("Welcome to YSOS Shell! Made by huang ye, student code 22336007");
    let mut root_dir = String::from("/APP/");
    println!("...");
    loop {
        print!("[{}]> ", root_dir);
        let input = stdin().read_line();
        let line: Vec<&str> = input.trim().split(' ').collect();

        // println!("line is {}", input);

        match line[0] {
            "cat" => {
                if line.len() < 2 {
                    println!("Usage: cat <file>");
                    continue;
                }
                service::cat(line[1], &mut root_dir);
            }
            "ls" => {
                service::ls(&root_dir);
            }
            "cd" => {
                if line.len() < 2 {
                    println!("Usage: cd <dir>");
                    continue;
                }
                service::cd(line[1], &mut root_dir);
            }
            "pwd" => {
                println!("{}", root_dir);
            }
            "echo" => {
                if line.len() < 2 {
                    println!("Usage: echo <string>");
                    continue;
                }
                println!("{}", line[1..].join(" "));
            }
            "exit" => {
                println!("Goodbye! ^_^");
                break;
            },
            "lsapp" => {
                service::list_app();
                continue;
            },
            "status" => {
                service::list_proc();
                continue;
            },
            "exec" => {
                if line.len() < 2 {
                    println!("Usage: exec <file>");
                    continue;
                }
                service::exec(line[1]);
                continue;
            },
            "help" => {
                service::help();
                continue;
            },
            _ => {
                println!("Unknown command: {}", line[0]);
            }
        }
    }

    233
}

entry!(main);