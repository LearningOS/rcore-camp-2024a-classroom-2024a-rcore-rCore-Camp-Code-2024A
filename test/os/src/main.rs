#![no_std]
#![no_main]
mod lang_items;
use lang_items::{sys_exit,print};

#[no_mangle]
extern "C" fn _start() {
    println!("Hello, world!");
    sys_exit(0);
}