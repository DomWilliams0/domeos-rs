use cpuio::Port;
use spin::Mutex;
use vga;

use core::ops::*;
use x86_64::instructions as instr;
use x86_64::structures::idt::{ExceptionStackFrame, Idt, PageFaultErrorCode};

type IOPort = Mutex<Port<u8>>;
lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        // default handlers
        idt.divide_by_zero.set_handler_fn(isr_panic_de);
        idt.debug.set_handler_fn(isr_panic_db);
        idt.non_maskable_interrupt.set_handler_fn(isr_panic_nmi);
        idt.breakpoint.set_handler_fn(isr_panic_bp);
        idt.overflow.set_handler_fn(isr_panic_of);
        idt.bound_range_exceeded.set_handler_fn(isr_panic_br);
        idt.invalid_opcode.set_handler_fn(isr_panic_ud);
        idt.device_not_available.set_handler_fn(isr_panic_nm);
        idt.double_fault.set_handler_fn(isr_panic_df);
        idt.invalid_tss.set_handler_fn(isr_panic_ts);
        idt.segment_not_present.set_handler_fn(isr_panic_np);
        idt.stack_segment_fault.set_handler_fn(isr_panic_ss);
        idt.general_protection_fault.set_handler_fn(isr_panic_gp);
        idt.page_fault.set_handler_fn(isr_panic_pf);
        idt.x87_floating_point.set_handler_fn(isr_panic_mf);
        idt.alignment_check.set_handler_fn(isr_panic_ac);
        idt.machine_check.set_handler_fn(isr_panic_mc);
        idt.simd_floating_point.set_handler_fn(isr_panic_xf);
        idt.virtualization.set_handler_fn(isr_panic_ve);
        idt.security_exception.set_handler_fn(isr_panic_sx);

        idt[32+0].set_handler_fn(handlers::clock);
        idt[32+1].set_handler_fn(handlers::kb);
        idt[32+12].set_handler_fn(handlers::mouse);

        idt
    };

    static ref PIC_MASTER_COMMAND: IOPort = unsafe{Mutex::new(Port::new(0x20))};
    static ref PIC_MASTER_DATA: IOPort = unsafe{Mutex::new(Port::new(0x21))};
    static ref PIC_SLAVE_COMMAND: IOPort = unsafe{Mutex::new(Port::new(0xA0))};
    static ref PIC_SLAVE_DATA: IOPort = unsafe{Mutex::new(Port::new(0xA1))};
    static ref KEYBOARD: IOPort = unsafe{Mutex::new(Port::new(0x60))};
    static ref PS2: IOPort = unsafe{Mutex::new(Port::new(0x64))};
}

fn enable_pics_and_remap_irqs() {
    let mut master_cmd = PIC_MASTER_COMMAND.lock();
    let mut master_data = PIC_MASTER_DATA.lock();
    let mut slave_cmd = PIC_SLAVE_COMMAND.lock();
    let mut slave_data = PIC_SLAVE_DATA.lock();

    // enable
    master_cmd.write(0x11);
    slave_cmd.write(0x11);

    // remap
    master_data.write(0x20);
    slave_data.write(0x28);
    master_data.write(0x01);
    slave_data.write(0x01);
    master_data.write(0x0);
    slave_data.write(0x0);
}

fn write_ps2(b: u8) {
    let mut ps2 = PS2.lock();
    // wait for bit 1 to be CLEAR
    while ps2.read().bitand(2) == 0x1 {}

    println!("starting writing to ps2");
    ps2.write(b);
    println!("done writing to ps2");
}

fn write_kb(b: u8) {
    // wait for bit 1
    while PS2.lock().read() & 2 == 0x1 {}
    KEYBOARD.lock().write(b);
}

fn read_ps2() -> u8 {
    PS2.lock().read()
}

/// bit 5 = for mouse
fn read_kb() -> u8 {
    // wait for bit 0
    while PS2.lock().read() & 1 != 0x1 {}
    KEYBOARD.lock().read()
}

fn start_ps2() {
    // All output to port 0x60 or 0x64 must be preceded by waiting for bit 1 (value=2) of port 0x64 to become clear.
    // Similarly, bytes cannot be read from port 0x60 until bit 0 (value=1) of port 0x64 is set.

    // mouse
    // enables aux input
    // does no harm

    write_ps2(0xa8);
    read_kb();

    write_ps2(0x20);
    let status = read_ps2();
    // set 1 and clear 5
    let status = status.bitor(1 << 1).bitand(0xff - (1 << 5));
    write_ps2(0x60);
    write_kb(status);

    let ack = read_kb();
    println!("status set ack = {:x}", ack);

    // enable
    write_kb(0xf4);
    let ack = read_kb();
    println!("enable ack = {:x}", ack);
}

/// true = mouse, false = kb, None = neither
/// TODO this is horrendous
fn ps2_ready() -> Option<bool> {
    let c = read_ps2();
    let data_present = c.bitand(0x1) == 1;
    let is_mouse = c.bitand(0x20) == 1;

    println!("{} {}", data_present, is_mouse);

    if data_present {
        Some(is_mouse)
    } else {
        None
    }
}

pub fn register() {
    enable_pics_and_remap_irqs();
    IDT.load();
    start_ps2();
}

// fn as_unit<T>(_: &mut T) -> () {}

macro_rules! irq_handler {
    ($no:expr, $name:ident, $body:block) => {
        // somehow need to generate a unique name
        // lazy_static! {
        //     static ref $name: () = as_unit(IDT[32+$no].set_handler_fn($name));
        // }

        pub extern "x86-interrupt" fn $name(_: &mut ExceptionStackFrame) {
            $body
                irq_clear($no >= 8);
        }
    };
}

fn irq_clear(slave: bool) {
    let mut master_cmd = PIC_MASTER_COMMAND.lock();
    let mut slave_cmd = PIC_SLAVE_COMMAND.lock();

    if slave {
        slave_cmd.write(0x20);
    }
    master_cmd.write(0x20);
}

mod handlers {
    use super::*;

    irq_handler!(0, clock, {
        // vga::get().set_colours(vga::Colour::Black, vga::Colour::White);
        //print!("clock ");
    });

    irq_handler!(1, kb, {
        println!("kb fired");
        //if !ps2_ready().unwrap_or(false) {
        //    let scancode = KEYBOARD.lock().read();
        //    vga::get().set_colours(vga::Colour::White, vga::Colour::Black);
        //    println!("scancode: {}", scancode);
        //}
    });

    irq_handler!(12, mouse, {
        println!("mouse fired");
        if ps2_ready().unwrap_or(false) {
            let data = KEYBOARD.lock().read();
            print!("mouse {}! ", data);
        }
    });
}

/// Disable interrupts and loop forever
pub fn panic() -> ! {
    unsafe {
        instr::interrupts::disable();
    }
    loop {}
}

fn isr_panic(_sf: &mut ExceptionStackFrame, irq: &'static str, error_code: Option<u64>) {
    vga::set_error_colours();
    print!("\nException: unhandled interrupt \"{}\"", irq);
    if let Some(ec) = error_code {
        print!(" (err:{})", ec);
    }
    println!(" - halting");
    panic();
}

// yuck
extern "x86-interrupt" fn isr_panic_de(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "divide by zero", None);
}
extern "x86-interrupt" fn isr_panic_db(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "debug", None);
}
extern "x86-interrupt" fn isr_panic_nmi(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "non-maskable interrupt", None);
}
extern "x86-interrupt" fn isr_panic_bp(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "breakpoint", None);
}
extern "x86-interrupt" fn isr_panic_of(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "overflow", None);
}
extern "x86-interrupt" fn isr_panic_br(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "bound range exceeded", None);
}
extern "x86-interrupt" fn isr_panic_ud(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "invalid opcode", None);
}
extern "x86-interrupt" fn isr_panic_nm(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "device not available", None);
}
extern "x86-interrupt" fn isr_panic_df(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "double fault", Some(ec));
}
extern "x86-interrupt" fn isr_panic_ts(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "invalid TSS", Some(ec));
}
extern "x86-interrupt" fn isr_panic_np(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "segment not present", Some(ec));
}
extern "x86-interrupt" fn isr_panic_ss(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "stack-segment fault", Some(ec));
}
extern "x86-interrupt" fn isr_panic_gp(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "general protection fault", Some(ec));
}
extern "x86-interrupt" fn isr_panic_pf(sf: &mut ExceptionStackFrame, ec: PageFaultErrorCode) {
    isr_panic(sf, "page fault", Some(ec.bits()));
}
extern "x86-interrupt" fn isr_panic_mf(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "x87 floating-point exception", None);
}
extern "x86-interrupt" fn isr_panic_ac(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "alignment check", Some(ec));
}
extern "x86-interrupt" fn isr_panic_mc(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "machine check", None);
}
extern "x86-interrupt" fn isr_panic_xf(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "SIMD floating-point exception", None);
}
extern "x86-interrupt" fn isr_panic_ve(sf: &mut ExceptionStackFrame) {
    isr_panic(sf, "virtualization exception", None);
}
extern "x86-interrupt" fn isr_panic_sx(sf: &mut ExceptionStackFrame, ec: u64) {
    isr_panic(sf, "security exception", Some(ec));
}
