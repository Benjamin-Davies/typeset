use std::fmt::{self, Write};

use glam::Vec2;

use crate::{char_map::CharMap, document::Style, text_layout::Line};

use super::cmap::MappedStr;

// A4 page size
pub const PAGE_WIDTH: f32 = 8.27 * 72.0;
pub const PAGE_HEIGHT: f32 = 11.69 * 72.0;

pub struct PageBuilder {
    content: String,
}

impl Default for PageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PageBuilder {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn begin_text(&mut self) -> Result<(), fmt::Error> {
        writeln!(self.content, "BT")
    }

    fn end_text(&mut self) -> Result<(), fmt::Error> {
        writeln!(self.content, "ET")
    }

    fn style(&mut self, style: &Style) -> Result<(), fmt::Error> {
        writeln!(self.content, "/{} {} Tf", style.font, style.font_size)
    }

    fn text_line_delta(&mut self, delta: Vec2) -> Result<(), fmt::Error> {
        writeln!(self.content, "{} {} Td", delta.x, delta.y)
    }

    pub fn text(&mut self, lines: &[Line], char_map: &CharMap) -> Result<(), fmt::Error> {
        self.begin_text()?;

        let mut current_style = Style::default();

        for line in lines {
            self.text_line_delta(line.delta)?;

            let Some(first_chunk) = line.chunks.first() else {
                continue;
            };

            if first_chunk.style != current_style {
                current_style = first_chunk.style;
                self.style(&current_style)?;
            }

            write!(self.content, "[")?;
            for chunk in &line.chunks {
                if chunk.style != current_style {
                    writeln!(self.content, "] TJ")?;
                    current_style = chunk.style;
                    self.style(&current_style)?;
                    write!(self.content, "[")?;
                }

                if chunk.left_adjust != 0.0 {
                    write!(
                        self.content,
                        "{}",
                        (1000.0 * chunk.left_adjust / chunk.style.font_size) as i32,
                    )?;
                }

                write!(self.content, "{}", MappedStr(&chunk.text, char_map))?;
            }
            writeln!(self.content, "] TJ")?;
        }
        self.end_text()?;

        Ok(())
    }

    pub fn build(self) -> String {
        self.content
    }
}
