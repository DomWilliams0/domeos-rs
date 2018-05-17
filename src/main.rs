#![no_std]
#![feature(lang_items)]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga = 0xb8000 as *const u8 as *mut u8;
    let pos = 30;
    unsafe {
        *vga.offset(pos) = 'X' as u8;
        *vga.offset(pos + 1) = 0xa;
    }

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
