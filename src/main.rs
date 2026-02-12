#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[link_section = ".multiboot_header"]
#[no_mangle]
pub static MULTIBOOT_HEADER: [u32; 3] = [
    0x1BADB002, 0x00, u32::wrapping_add(0x1BADB002, 0x00).wrapping_neg(),
];

const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;

// --- OS STRUCTURES ---
#[derive(PartialEq, Eq, Clone, Copy)]
enum App { Desktop, Notepad, FileManager, Snake, Dino, Pong, Shutdown }

static mut CURRENT_APP: App = App::Desktop;
static mut PREVIOUS_APP: App = App::Shutdown;
static mut CURSOR_X: i16 = 40;
static mut CURSOR_Y: i16 = 12;

// Colors (VGA Palette)
const COLOR_DESKTOP: u8 = 0x1F; // Blue
const COLOR_SNAKE_BG: u8 = 0x20; // Dark Green
const COLOR_DINO_BG: u8 = 0x70;  // Light Grey (Desert)
const COLOR_PONG_BG: u8 = 0x40;  // Dark Red

// Storage
static mut FILE_CONTENT: [u8; 64] = [0; 64];
static mut TYPING_IDX: usize = 0;

// Game States
static mut SNAKE_POS: (i16, i16) = (40, 12);
static mut SNAKE_DIR: (i16, i16) = (1, 0);
static mut DINO_Y: i16 = 20;
static mut DINO_VEL: i16 = 0;
static mut OBSTACLE_X: i16 = 79;
static mut PADDLE_Y: i16 = 10;
static mut BALL_POS: (i16, i16) = (40, 12);
static mut BALL_DIR: (i16, i16) = (1, 1);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {
        unsafe {
            if CURRENT_APP != PREVIOUS_APP {
                setup_screen_background();
                PREVIOUS_APP = CURRENT_APP;
            }
            handle_input_logic();
            run_active_app();
            // CPU delay for smooth framerate
            for _ in 0..380000 { core::hint::black_box(0); }
        }
    }
}

unsafe fn setup_screen_background() {
    match CURRENT_APP {
        App::Desktop => {
            clear_screen(COLOR_DESKTOP);
            draw_rect(0, 0, 80, 1, 0x70);
            print_string(" JABBERR OS v1.4 - HIGH CONTRAST ", 1, 0, 0x70);
            print_string("[WRITE] [FILES] [SNAKE] [DINO] [PONG]", 2, 2, 0x1E);
        },
        App::Notepad => clear_screen(0x0F),
        App::FileManager => clear_screen(0x1F),
        App::Snake => clear_screen(COLOR_SNAKE_BG),
        App::Dino => clear_screen(COLOR_DINO_BG),
        App::Pong => clear_screen(COLOR_PONG_BG),
        _ => clear_screen(0x00),
    }
}

unsafe fn run_active_app() {
    match CURRENT_APP {
        App::Snake => tick_snake_engine(),
        App::Dino => tick_dino_engine(),
        App::Pong => tick_pong_engine(),
        App::Notepad => {
            draw_rect(0, 0, 80, 1, 0x08);
            print_string(" NOTEPAD - [ESC] TO SAVE ", 1, 0, 0x0F);
            print_string(core::str::from_utf8_unchecked(&FILE_CONTENT), 2, 2, 0x08);
        },
        App::FileManager => {
            print_string("--- DISK DRIVE A: ---", 28, 2, 0x1F);
            print_string("FILE: note1.txt", 5, 5, 0x1E);
            print_string(core::str::from_utf8_unchecked(&FILE_CONTENT), 5, 8, 0x1F);
        },
        _ => {}
    }
}

unsafe fn handle_input_logic() {
    if (inb(0x64) & 1) != 0 {
        let scancode = inb(0x60);
        if scancode < 0x80 {
            match scancode {
                0x01 => CURRENT_APP = App::Desktop,
                0x48 => { // UP
                    if CURRENT_APP == App::Desktop { if CURSOR_Y > 0 { CURSOR_Y -= 1; } }
                    else if CURRENT_APP == App::Snake { SNAKE_DIR = (0, -1); }
                    else if CURRENT_APP == App::Dino && DINO_Y == 20 { DINO_VEL = -2; }
                    else if CURRENT_APP == App::Pong && PADDLE_Y > 1 { PADDLE_Y -= 1; }
                },
                0x50 => { // DOWN
                    if CURRENT_APP == App::Desktop { if CURSOR_Y < 24 { CURSOR_Y += 1; } }
                    else if CURRENT_APP == App::Snake { SNAKE_DIR = (0, 1); }
                    else if CURRENT_APP == App::Pong && PADDLE_Y < 21 { PADDLE_Y += 1; }
                },
                0x4B => { // LEFT
                    if CURRENT_APP == App::Desktop { if CURSOR_X > 0 { CURSOR_X -= 1; } }
                    else if CURRENT_APP == App::Snake { SNAKE_DIR = (-1, 0); }
                },
                0x4D => { // RIGHT
                    if CURRENT_APP == App::Desktop { if CURSOR_X < 79 { CURSOR_X += 1; } }
                    else if CURRENT_APP == App::Snake { SNAKE_DIR = (1, 0); }
                },
                0x1C => if CURRENT_APP == App::Desktop { check_app_launch(); },
                _ => if CURRENT_APP == App::Notepad { handle_typing(scancode); }
            }
            update_vga_cursor(CURSOR_X as u16, CURSOR_Y as u16);
        }
    }
}

unsafe fn check_app_launch() {
    if CURSOR_Y == 2 {
        if CURSOR_X >= 2 && CURSOR_X <= 8 { CURRENT_APP = App::Notepad; }
        else if CURSOR_X >= 10 && CURSOR_X <= 16 { CURRENT_APP = App::FileManager; }
        else if CURSOR_X >= 18 && CURSOR_X <= 24 { CURRENT_APP = App::Snake; }
        else if CURSOR_X >= 26 && CURSOR_X <= 32 { CURRENT_APP = App::Dino; }
        else if CURSOR_X >= 34 && CURSOR_X <= 40 { CURRENT_APP = App::Pong; }
    }
}

// --- ENGINES (FIXED COLORS) ---

unsafe fn tick_snake_engine() {
    // Erase with Snake BG color, not Black
    print_string(" ", SNAKE_POS.0 as isize, SNAKE_POS.1 as isize, COLOR_SNAKE_BG);
    SNAKE_POS.0 += SNAKE_DIR.0; SNAKE_POS.1 += SNAKE_DIR.1;
    if SNAKE_POS.0 < 0 || SNAKE_POS.0 > 79 || SNAKE_POS.1 < 1 { SNAKE_POS = (40, 12); }
    print_string("O", SNAKE_POS.0 as isize, SNAKE_POS.1 as isize, COLOR_SNAKE_BG | 0x0E);
}

unsafe fn tick_dino_engine() {
    print_string(" ", 5, DINO_Y as isize, COLOR_DINO_BG);
    print_string(" ", OBSTACLE_X as isize, 20, COLOR_DINO_BG);
    DINO_Y += DINO_VEL;
    if DINO_Y < 17 { DINO_VEL = 1; }
    if DINO_Y > 20 { DINO_Y = 20; DINO_VEL = 0; }
    OBSTACLE_X -= 1;
    if OBSTACLE_X < 0 { OBSTACLE_X = 79; }
    print_string("R", 5, DINO_Y as isize, COLOR_DINO_BG | 0x00); // Black Dino on Grey
    print_string("#", OBSTACLE_X as isize, 20, COLOR_DINO_BG | 0x04); // Red Cactus
}

unsafe fn tick_pong_engine() {
    print_string(" ", 2, PADDLE_Y as isize - 1, COLOR_PONG_BG);
    print_string(" ", 2, PADDLE_Y as isize + 3, COLOR_PONG_BG);
    print_string(" ", BALL_POS.0 as isize, BALL_POS.1 as isize, COLOR_PONG_BG);
    BALL_POS.0 += BALL_DIR.0; BALL_POS.1 += BALL_DIR.1;
    if BALL_POS.1 <= 1 || BALL_POS.1 >= 24 { BALL_DIR.1 = -BALL_DIR.1; }
    if BALL_POS.0 >= 79 { BALL_DIR.0 = -BALL_DIR.0; }
    if BALL_POS.0 == 3 && BALL_POS.1 >= PADDLE_Y && BALL_POS.1 <= PADDLE_Y + 2 {
        BALL_DIR.0 = -BALL_DIR.0;
    } else if BALL_POS.0 <= 0 { BALL_POS = (40, 12); }
    draw_rect(2, PADDLE_Y as isize, 1, 3, 0x0F);
    print_string("*", BALL_POS.0 as isize, BALL_POS.1 as isize, COLOR_PONG_BG | 0x0E);
}

// --- UTILITIES ---

unsafe fn handle_typing(s: u8) {
    let c = match s {
        0x10..=0x19 => "qwertyuiop".as_bytes()[(s-0x10) as usize] as char,
        0x1E..=0x26 => "asdfghjkl".as_bytes()[(s-0x1E) as usize] as char,
        0x2C..=0x32 => "zxcvbnm".as_bytes()[(s-0x2C) as usize] as char,
        0x39 => ' ', _ => '\0'
    };
    if s == 0x0E && TYPING_IDX > 0 { TYPING_IDX -= 1; FILE_CONTENT[TYPING_IDX] = 0; }
    else if c != '\0' && TYPING_IDX < 63 { FILE_CONTENT[TYPING_IDX] = c as u8; TYPING_IDX += 1; }
}

fn update_vga_cursor(x: u16, y: u16) {
    let pos = y * 80 + x;
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Fu8);
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") (pos & 0xFF) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Eu8);
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") ((pos >> 8) & 0xFF) as u8);
    }
}

fn inb(port: u16) -> u8 {
    let v: u8; unsafe { core::arch::asm!("in al, dx", out("al") v, in("dx") port); } v
}

fn print_string(text: &str, x: isize, y: isize, color: u8) {
    for (i, b) in text.bytes().enumerate() {
        let pos = y * 80 + x + i as isize;
        if pos >= 0 && pos < 2000 {
            unsafe { core::ptr::write_volatile(VGA_BUFFER.offset(pos), (color as u16) << 8 | (b as u16)); }
        }
    }
}

fn clear_screen(c: u8) {
    let f = (c as u16) << 8 | (' ' as u16);
    for i in 0..2000 { unsafe { core::ptr::write_volatile(VGA_BUFFER.offset(i), f); } }
}

fn draw_rect(x: isize, y: isize, w: isize, h: isize, c: u8) {
    for row in 0..h { for col in 0..w {
        let pos = (y + row) * 80 + (x + col);
        if pos < 2000 { unsafe { core::ptr::write_volatile(VGA_BUFFER.offset(pos), (c as u16) << 8 | (' ' as u16)); } }
    } }
}

#[panic_handler] fn panic(_info: &PanicInfo) -> ! { loop {} }