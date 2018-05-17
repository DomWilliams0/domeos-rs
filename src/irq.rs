use cpuio::Port;
use spin::Mutex;
use vga;

use x86_64::instructions as instr;
use x86_64::structures::idt::{Idt, ExceptionStackFrame, PageFaultErrorCode};

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

        idt[32].set_handler_fn(irq_clock);
        idt[33].set_handler_fn(irq_kb);

        idt
    };

    static ref PIC_MASTER_COMMAND: IOPort = unsafe{Mutex::new(Port::new(0x20))};
    static ref PIC_MASTER_DATA: IOPort = unsafe{Mutex::new(Port::new(0x21))};
    static ref PIC_SLAVE_COMMAND: IOPort = unsafe{Mutex::new(Port::new(0xA0))};
    static ref PIC_SLAVE_DATA: IOPort = unsafe{Mutex::new(Port::new(0xA1))};
    static ref KEYBOARD: IOPort = unsafe{Mutex::new(Port::new(0x60))};
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

pub fn register() {
    enable_pics_and_remap_irqs();

    IDT.load();
}

fn irq_clear() {
    let mut master_cmd = PIC_MASTER_COMMAND.lock();
    let mut slave_cmd = PIC_SLAVE_COMMAND.lock();

    slave_cmd.write(0x20);
    master_cmd.write(0x20);
}

extern "x86-interrupt" fn irq_clock(_: &mut ExceptionStackFrame) {
    vga::get().set_colours(vga::Colour::Black, vga::Colour::White);
    print!("clock ");
    irq_clear();
}

extern "x86-interrupt" fn irq_kb(_: &mut ExceptionStackFrame) {
    let scancode = KEYBOARD.lock().read();
    vga::get().set_colours(vga::Colour::White, vga::Colour::Black);
    println!("scancode: {}", scancode);
    irq_clear();
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
