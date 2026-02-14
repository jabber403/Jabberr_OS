use core::ptr::write_volatile;

pub struct Canvas {
    pub ptr: *mut u32,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    // Draws a solid colored block
    pub fn draw_rect(&self, x1: usize, y1: usize, x2: usize, y2: usize, color: u32) {
        for y in y1..y2 {
            for x in x1..x2 {
                if x < self.width && y < self.height {
                    unsafe {
                        write_volatile(self.ptr.add(y * self.width + x), color);
                    }
                }
            }
        }
    }

    // Draws a "Jabberr-style" Window
    pub fn draw_window(&self, x: usize, y: usize, w: usize, h: usize) {
        let title_height = 25;
        // Window Shadow
        self.draw_rect(x + 3, y + 3, x + w + 3, y + h + 3, 0xFF17202A);
        // Main Body (Light Grey)
        self.draw_rect(x, y, x + w, y + h, 0xFFBDC3C7);
        // Title Bar (Jabberr Blue)
        self.draw_rect(x, y, x + w, y + title_height, 0xFF2980B9);
        // "Close" Button (Red Square)
        self.draw_rect(x + 5, y + 5, x + 20, y + 20, 0xFFC0392B);
    }
}