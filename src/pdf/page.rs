use std::io::Write;

use crate::{font::Font, text_layout::compute_x_positions};

// A4 page size
pub const PAGE_WIDTH: f32 = 8.27 * 72.0;
pub const PAGE_HEIGHT: f32 = 11.69 * 72.0;

pub struct PageBuilder {
    content: Vec<u8>,
}

impl PageBuilder {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn paragraph(mut self, font: &Font, font_size: f32, s: &str) -> Self {
        // Use our own spacing calculations
        let glyphs = compute_x_positions(font, font_size, s);
        for (c, x) in glyphs {
            write!(
                self.content,
                "BT\n/{} 12 Tf\n{} {} Td\n({}) Tj\nET\n",
                font.ps_name,
                72.0 + x,
                1.5 * 72.0,
                c,
            )
            .unwrap();
        }

        // Use the built-in spacing calculations
        write!(
            self.content,
            "BT\n/{} 12 Tf\n72.0 72.0 Td\n({}) Tj\nET\n",
            font.ps_name, s,
        )
        .unwrap();

        self
    }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
