#![no_std]
#![feature(lang_items)]
#![feature(const_fn)]
#![no_main]
#![allow(dead_code)]

extern crate volatile;

mod vga;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut s = vga::Screen::default();
    s.write_string("one\ntwo\nthree\n");
    s.write_string("this is a very long message that should wrap at the edges if it is long enough, which it is");
    loop {}
}

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
