#![no_std]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code, unused_macros)]

extern crate cpuio;
extern crate spin;
extern crate volatile;
extern crate x86_64;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod vga;
mod irq;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    irq::register();

    println!("one\ntwo\nthree = {}, {}, {}", 1, 2, 22.6 / 7.0);
    println!("this is a very long message that should wrap at the edges if it is long enough, which it is");

    loop {}
}

#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    msg: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    column: u32,
) -> ! {
    vga::set_error_colours();
    println!("\nRUST PANIC at {}:{}:{} - {}", file, line, column, msg);
    loop {}
}
