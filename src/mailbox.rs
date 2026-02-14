use core::ptr::{read_volatile, write_volatile};

pub const MAIL_BASE: u32 = 0x2000B880;
pub const MAIL_READ: *mut u32 = (MAIL_BASE + 0x00) as *mut u32;
pub const MAIL_STATUS: *mut u32 = (MAIL_BASE + 0x18) as *mut u32;
pub const MAIL_WRITE: *mut u32 = (MAIL_BASE + 0x20) as *mut u32;

pub fn send_message(ptr: u32) {
    unsafe {
        // Wait until mailbox is ready to receive
        while read_volatile(MAIL_STATUS) & 0x80000000 != 0 {
            core::arch::asm!("nop");
        }
        
        // Write message to channel 8 (Properties)
        write_volatile(MAIL_WRITE, ptr | 8);
        
        // Wait for the specific response
        loop {
            while read_volatile(MAIL_STATUS) & 0x40000000 != 0 {
                core::arch::asm!("nop");
            }
            if read_volatile(MAIL_READ) == (ptr | 8) {
                return;
            }
        }
    }
}