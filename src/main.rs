#![no_std]
#![feature(lang_items)]
#![feature(const_fn)]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, unused_macros)]

extern crate volatile;
extern crate spin;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod vga;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("one\ntwo\nthree = {}, {}, {}", 1, 2, 22.6/7.0);
    println!("this is a very long message that should wrap at the edges if it is long enough, which it is");

    vga::get().set_colours(vga::Colour::Pink, vga::Colour::White);
    println!("bye bye");
    loop {}
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
    _column: u32,
) -> ! {
    loop {}
}
