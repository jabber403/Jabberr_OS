#![no_std]
#![no_main]

mod mailbox;
mod gui;

use core::panic::PanicInfo;
use core::arch::global_asm;
use gui::Canvas;

// This assembly sets up the stack pointer so the CPU can run Rust code
global_asm!(
    ".cpu arm1176jzf-s",
    ".section .text.boot",
    ".global _start",
    "_start:",
    "mov sp, #0x8000",
    "bl kernel_main",
    "hang: b hang"
);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    #[repr(align(16))]
    struct Mbox { data: [u32; 36] }
    let mut m = Mbox { data: [0; 36] };

    // Request a 640x480 frame buffer from the GPU
    m.data[0] = 35 * 4; 
    m.data[2] = 0x48003; m.data[3] = 8; m.data[4] = 8; m.data[5] = 640; m.data[6] = 480;
    m.data[7] = 0x48004; m.data[8] = 8; m.data[9] = 8; m.data[10] = 640; m.data[11] = 480;
    m.data[12] = 0x48005; m.data[13] = 4; m.data[14] = 4; m.data[15] = 32;
    m.data[16] = 0x40001; m.data[17] = 8; m.data[18] = 8; m.data[19] = 4096; m.data[20] = 0;
    m.data[21] = 0;

    // Send the request to the Mailbox (Channel 8)
    mailbox::send_message((&m as *const Mbox as u32) | 0x40000000);

    let fb_ptr = m.data[19] & 0x3FFFFFFF;
    if fb_ptr != 0 {
        let canvas = Canvas { ptr: fb_ptr as *mut u32, width: 640, height: 480 };

        // Draw the Desktop
        canvas.draw_rect(0, 0, 640, 480, 0xFF2E4053); // Blue-grey background
        canvas.draw_rect(0, 450, 640, 480, 0xFF1C2833); // Dark taskbar

        // Draw some OS Windows
        canvas.draw_window(50, 50, 300, 200, 0xFF2980B9);  // "System" window
        canvas.draw_window(380, 100, 200, 150, 0xFF8E44AD); // "App" window
    }

    loop {}
}

// This is the missing piece the error was complaining about!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use core::ptr::{read_volatile, write_volatile};

pub const MAIL_BASE: u32 = 0x2000B880;
pub const MAIL_READ: *mut u32 = (MAIL_BASE + 0x00) as *mut u32;
pub const MAIL_STATUS: *mut u32 = (MAIL_BASE + 0x18) as *mut u32;
pub const MAIL_WRITE: *mut u32 = (MAIL_BASE + 0x20) as *mut u32;

pub fn send_message(ptr: u32) {
    unsafe {
        while read_volatile(MAIL_STATUS) & 0x80000000 != 0 {
            core::arch::asm!("nop");
        }
        write_volatile(MAIL_WRITE, ptr | 8);
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