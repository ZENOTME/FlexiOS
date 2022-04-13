#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;


use alloc::string::String;

#[no_mangle]
pub fn main() -> i32 {
    let mut s = String::new();
    loop {
        let ch=user_lib::getchar() as char;
        if ch=='\n'{
            if s.as_str()=="exit"{
                break;
            }
            println!("{}", s);
            s.clear();
        }else{
            s.push(ch);
        }
    }
    0
}