#![no_std]
#![no_main]

mod mailbox;
mod gui;

use core::panic::PanicInfo;
use core::arch::global_asm;
use gui::Canvas;

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

    // Mailbox setup for 640x480 32-bit color
    m.data[0] = 35 * 4; 
    m.data[2] = 0x48003; m.data[3] = 8; m.data[4] = 8; m.data[5] = 640; m.data[6] = 480;
    m.data[7] = 0x48004; m.data[8] = 8; m.data[9] = 8; m.data[10] = 640; m.data[11] = 480;
    m.data[12] = 0x48005; m.data[13] = 4; m.data[14] = 4; m.data[15] = 32;
    m.data[16] = 0x40001; m.data[17] = 8; m.data[18] = 8; m.data[19] = 4096; m.data[20] = 0;
    m.data[21] = 0;

    mailbox::send_message((&m as *const Mbox as u32) | 0x40000000);

    let fb_ptr = m.data[19] & 0x3FFFFFFF;
    if fb_ptr != 0 {
        let canvas = Canvas { ptr: fb_ptr as *mut u32, width: 640, height: 480 };

        // Desktop Background
        canvas.draw_rect(0, 0, 640, 480, 0xFF2E4053);
        // Taskbar
        canvas.draw_rect(0, 450, 640, 480, 0xFF1C2833);

        // Open some OS Windows
        canvas.draw_window(50, 50, 300, 200, 0xFF2980B9);  // Main System Window
        canvas.draw_window(380, 100, 200, 150, 0xFF8E44AD); // Secondary App
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }