use bit_field::BitField;
use core::mem::size_of;

#[repr(C)]
#[derive(Debug)]
struct InterruptStackContext {
    rip: u64,
    cs: u64,
    flags: u64,
    rsp: u64,
    ss: u64,
}

const IDT_ENTRY_COUNT: usize = 16;

#[derive(Debug)]
pub struct Idt {
    entries: [IdtEntry; IDT_ENTRY_COUNT],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IdtEntry {
    ptr_low: u16,
    gdt_selector: u16,
    options: u16,
    ptr_mid: u16,
    ptr_high: u32,
    reserved: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum IdtEntryOption {
    Present,
    NotPresent,

    DisableInterrupts,
    EnableInterrupts,

    User,
    Supervisor,

    StackIndex(u16),
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    addr: u64,
}

impl Default for IdtEntry {
    fn default() -> Self {
        let mut entry = IdtEntry::new();
        entry.set_option(IdtEntryOption::Present);
        entry.set_option(IdtEntryOption::DisableInterrupts);
        entry
    }
}

impl IdtEntry {
    fn new() -> Self {
        let mut opts = 0;
        opts.set_bits(9..12, 0b111); // reserved
        IdtEntry {
            ptr_low: 0,
            gdt_selector: 0,
            options: opts,
            ptr_mid: 0,
            ptr_high: 0,
            reserved: 0,
        }
    }

    pub fn set_option(&mut self, option: IdtEntryOption) {
        unsafe {
            match option {
                IdtEntryOption::Present => self.options.set_bit(15, true),
                IdtEntryOption::NotPresent => self.options.set_bit(15, false),

                IdtEntryOption::DisableInterrupts => self.options.set_bit(8, false),
                IdtEntryOption::EnableInterrupts => self.options.set_bit(8, true),

                IdtEntryOption::User => self.options.set_bits(13..15, 3),
                IdtEntryOption::Supervisor => self.options.set_bits(13..15, 0),

                IdtEntryOption::StackIndex(i) => self.options.set_bits(0..3, i),
            };
        }
    }
}

impl Idt {
    pub fn new() -> Self {
        Self {
            entries: [IdtEntry::new(); IDT_ENTRY_COUNT],
        }
    }

    pub fn install(&'static self) {
        unsafe { IdtDescriptor::new(self).lidt() }
    }
}

impl IdtDescriptor {
    fn new(idt: &'static Idt) -> Self {
        Self {
            limit: (size_of::<Idt>() - 1) as u16,
            addr: idt as *const _ as u64,
        }
    }

    unsafe fn lidt(self) {
        asm!("lidt ($0)" :: "r" (&self) : "memory")
    }
}
