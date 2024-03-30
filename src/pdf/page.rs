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

        let mut current_style = Style::default();

        for line in lines {
            self.text_line_delta(line.delta);

            let Some(first_chunk) = line.chunks.first() else {
                continue;
            };

            if first_chunk.style != current_style {
                current_style = first_chunk.style;
                self.style(&current_style);
            }

            write!(self.content, "[").unwrap();
            for chunk in &line.chunks {
                if chunk.style != current_style {
                    write!(self.content, "] TJ\n").unwrap();
                    current_style = chunk.style;
                    self.style(&current_style);
                    write!(self.content, "[").unwrap();
                }

                write!(
                    self.content,
                    "{}({})",
                    (1000.0 * chunk.left_adjust / chunk.style.font_size) as i32,
                    chunk.text,
                )
                .unwrap();
            }
            write!(self.content, "] TJ\n").unwrap();
        }
        self.end_text();

        self
    }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
