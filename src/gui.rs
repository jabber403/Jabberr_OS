use core::ptr::write_volatile;

pub struct Canvas {
    pub ptr: *mut u32,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
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

    pub fn draw_window(&self, x: usize, y: usize, w: usize, h: usize, title_color: u32) {
        // Shadow
        self.draw_rect(x + 4, y + 4, x + w + 4, y + h + 4, 0xFF1B2631);
        // Window Body
        self.draw_rect(x, y, x + w, y + h, 0xFFD5DBDB);
        // Title Bar
        self.draw_rect(x, y, x + w, y + 25, title_color);
        // Close Button
        self.draw_rect(x + 5, y + 5, x + 20, y + 20, 0xFFE74C3C);
    }
}