#![no_std]
#![no_main]


#[macro_use]
extern crate user_lib;


#[no_mangle]
pub fn main() -> i32 {
    let mut buf:[u8;1];
    loop{
        let ch=user_lib::getchar();
        print!("{}",ch as char);
    }
}