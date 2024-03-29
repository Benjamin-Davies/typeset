use std::io::Write;

use crate::{font::Font, text_layout::layout_paragraphs};

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
        let paragraphs = layout_paragraphs(font, font_size, s, PAGE_WIDTH - 2.0 * 72.0);

        let font_scale = font_size / font.face.units_per_em() as f32;
        let ascender_pt = font.face.ascender() as f32 * font_scale;
        let line_height = font.face.line_gap() + font.face.ascender() - font.face.descender();
        let line_height_pt = line_height as f32 * font_scale;

        let x = 72.0;
        let mut y = PAGE_HEIGHT - 72.0 - ascender_pt;

        for paragraph in paragraphs {
            write!(self.content, "BT\n").unwrap();
            write!(self.content, "/{} {} Tf\n", font.ps_name, font_size).unwrap();
            write!(self.content, "{} TL\n", line_height_pt).unwrap();
            write!(self.content, "{} {} Td\n", x, y).unwrap();

            for line in paragraph {
                write!(self.content, "({}) Tj\n", line).unwrap();
                write!(self.content, "T*\n").unwrap();
                y -= line_height_pt;
            }

            write!(self.content, "ET\n").unwrap();
            y -= font_size;
        }

        self
    }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
