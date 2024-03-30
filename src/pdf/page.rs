use std::io::Write;

use glam::Vec2;

use crate::{document::Style, text_layout::Line};

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

    fn begin_text(&mut self) {
        write!(self.content, "BT\n").unwrap();
    }

    fn end_text(&mut self) {
        write!(self.content, "ET\n").unwrap();
    }

    fn style(&mut self, style: &Style) {
        write!(self.content, "/{} {} Tf\n", style.font, style.font_size).unwrap();
    }

    fn text_line_delta(&mut self, delta: Vec2) {
        write!(self.content, "{} {} Td\n", delta.x, delta.y).unwrap();
    }

    pub fn text(mut self, lines: &[Line]) -> Self {
        self.begin_text();
        for line in lines {
            self.text_line_delta(line.delta);

            let Some(first_chunk) = line.chunks.first() else {
                continue;
            };
            let mut current_style = first_chunk.style;
            self.style(&current_style);

            write!(self.content, "[").unwrap();
            for chunk in &line.chunks {
                if chunk.style != current_style {
                    write!(self.content, "] TJ\n").unwrap();
                    current_style = chunk.style;
                    self.style(&current_style);
                    write!(self.content, "[").unwrap();
                }

                write!(self.content, "{}({})", chunk.left_adjust, chunk.text).unwrap();
            }
            write!(self.content, "] TJ\n").unwrap();
        }
        self.end_text();

        self
    }

    // pub fn paragraph(mut self, font: &Font, font_size: f32, s: &str) -> Self {
    //     let font_scale = font_size / font.face.units_per_em() as f32;
    //     let ascender_pt = font.face.ascender() as f32 * font_scale;
    //     let line_height_pt = font.line_height() as f32 * font_scale;

    //     let x = 72.0;
    //     let mut y = PAGE_HEIGHT - 72.0 - ascender_pt;
    //     let width = PAGE_WIDTH - 2.0 * 72.0;

    //     let paragraphs = layout_paragraphs(font, font_size, s, width);

    //     for paragraph in paragraphs {
    //         write!(self.content, "BT\n").unwrap();
    //         write!(self.content, "/{} {} Tf\n", font.ps_name, font_size).unwrap();
    //         write!(self.content, "{} TL\n", line_height_pt).unwrap();
    //         write!(self.content, "{} {} Td\n", x, y).unwrap();

    //         for (i, line) in paragraph.iter().enumerate() {
    //             if i < paragraph.len() - 1 {
    //                 let (words, space) = justify_line(font, font_size, line, width);

    //                 write!(self.content, "[").unwrap();
    //                 for word in words {
    //                     write!(self.content, "({}){}", word, -space).unwrap();
    //                 }
    //                 write!(self.content, "] TJ\n").unwrap();
    //             } else {
    //                 write!(self.content, "({}) Tj\n", line).unwrap();
    //             }

    //             write!(self.content, "T*\n").unwrap();
    //             y -= line_height_pt;
    //         }

    //         write!(self.content, "ET\n").unwrap();
    //         y -= font_size;
    //     }

    //     self
    // }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
