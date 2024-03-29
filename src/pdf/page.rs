use std::io::Write;

use crate::font::FONT_PS_NAME;

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

    pub fn paragraph(mut self, text: &str) -> Self {
        // TODO: use our own spacing calculations
        write!(
            self.content,
            "BT\n/{FONT_PS_NAME} 12 Tf\n72 72 Td\n({}) Tj\nET\n",
            text
        )
        .unwrap();
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
