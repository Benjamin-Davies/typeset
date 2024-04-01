use std::collections::BTreeMap;

use glam::{vec2, Vec2};

use crate::{
    document::{Block, Document, Inline, Style, TextAlign},
    font::{Font, TextMetrics},
};

const PARAGRAPH_GAP: f32 = 12.0;

#[derive(Debug, Default, Clone)]
pub struct Page<'a> {
    pub lines: Vec<Line<'a>>,
}

#[derive(Debug, Clone)]
pub struct Line<'a> {
    pub chunks: Vec<Chunk<'a>>,
    pub text_metrics: TextMetrics,
    pub text_total_width: f32,
    pub delta: Vec2,
}

#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    pub text: &'a str,
    pub style: Style<'a>,
    pub text_metrics: TextMetrics,
    pub width: f32,
    pub is_whitespace: bool,
    pub left_adjust: f32,
}

pub fn layout_document<'a>(document: &Document<'a>) -> Vec<Page<'a>> {
    let target_width = document.page_size.x - 2.0 * document.margin;
    let target_height = document.page_size.y - 2.0 * document.margin;

    let mut lines = Vec::new();
    for block in &document.blocks {
        let mut block_lines = layout_block(&document.fonts, target_width, block);

        if let Some(first_line) = block_lines.first_mut() {
            first_line.delta.y -= PARAGRAPH_GAP;
            first_line.text_metrics.line_gap += PARAGRAPH_GAP;
        }

        lines.append(&mut block_lines);
    }

    let mut pages = Vec::new();
    let mut current_page = Page::default();
    let mut current_page_height = 0.0;
    for line in lines {
        let mut line_height = line.text_metrics.line_height();
        if current_page.lines.is_empty() {
            line_height -= line.text_metrics.line_gap;
        }

        if current_page_height + line_height > target_height {
            pages.push(current_page);
            current_page = Page::default();
            current_page_height = 0.0;
        }

        current_page.lines.push(line);
        current_page_height += line_height;
    }
    pages.push(current_page);

    for page in &mut pages {
        if let Some(first_line) = page.lines.first_mut() {
            first_line.delta.x = document.margin;
            first_line.delta.y =
                document.page_size.y - document.margin - first_line.text_metrics.ascent;
        }
    }

    pages
}

fn layout_block<'a>(
    fonts: &BTreeMap<&str, &'a Font>,
    target_width: f32,
    block: &Block<'a>,
) -> Vec<Line<'a>> {
    match block {
        Block::Text(block) => {
            // Split block into chunks
            let chunks = block
                .inlines
                .iter()
                .flat_map(|inline| chunk_inline(fonts, inline))
                .collect::<Vec<_>>();

            // Organise chunks into lines
            let mut lines = Vec::<Line>::new();
            let mut line_start = 0;
            let mut possible_break = 0;
            let mut width_to_break = 0.0;
            let mut x = 0.0;
            let mut current_line_text_metrics = TextMetrics::default();
            for (i, chunk) in chunks.iter().enumerate() {
                if chunk.is_whitespace && !chunks[i.saturating_sub(1)].is_whitespace {
                    width_to_break = x;
                    possible_break = i;
                }

                if possible_break > line_start && x - chunk.left_adjust + chunk.width > target_width
                {
                    let mut line_spacing =
                        current_line_text_metrics.line_gap + current_line_text_metrics.ascent;
                    if let Some(prev_line) = lines.last() {
                        line_spacing -= prev_line.text_metrics.descent;
                    }

                    let line = Line {
                        chunks: chunks[line_start..possible_break].to_vec(),
                        text_metrics: current_line_text_metrics,
                        text_total_width: width_to_break,
                        delta: vec2(0.0, -line_spacing),
                    };
                    lines.push(line);

                    line_start = possible_break;
                    x = chunk.width;
                    current_line_text_metrics = chunk.text_metrics;

                    while let Some(next) = chunks.get(line_start) {
                        if next.is_whitespace {
                            line_start += 1;
                        } else {
                            break;
                        }
                    }
                } else if i >= line_start {
                    if i == line_start {
                        x = 0.0;
                    }
                    x += chunk.width;
                    current_line_text_metrics = current_line_text_metrics.max(chunk.text_metrics);
                }
            }

            // Add the last line if there are any chunks left
            if line_start < chunks.len() {
                let mut line_spacing =
                    current_line_text_metrics.line_gap + current_line_text_metrics.ascent;
                if let Some(prev_line) = lines.last() {
                    line_spacing -= prev_line.text_metrics.descent;
                }

                let line = Line {
                    chunks: chunks[line_start..].to_vec(),
                    text_metrics: current_line_text_metrics,
                    text_total_width: x,
                    delta: vec2(0.0, -line_spacing),
                };
                lines.push(line);
            }

            #[cfg(debug_assertions)]
            for line in &lines {
                debug_assert_eq!(
                    line.chunks.iter().map(|c| c.width).sum::<f32>(),
                    line.text_total_width,
                );
            }

            // Adjust alignment
            match block.align {
                TextAlign::Left => {} // Do nothing
                TextAlign::Center => {
                    for line in &mut lines {
                        let remaining_width = target_width - line.text_total_width;
                        if let Some(first_chunk) = line.chunks.first_mut() {
                            first_chunk.left_adjust = -0.5 * remaining_width;
                        }
                    }
                }
                TextAlign::Right => {
                    for line in &mut lines {
                        let remaining_width = target_width - line.text_total_width;
                        if let Some(first_chunk) = line.chunks.first_mut() {
                            first_chunk.left_adjust = -remaining_width;
                        }
                    }
                }
                TextAlign::Justify => {
                    let line_count = lines.len();
                    for line in &mut lines[0..line_count.saturating_sub(1)] {
                        let remaining_width = target_width - line.text_total_width;

                        let mut num_whitespace_gaps = 0;
                        for i in 1..line.chunks.len() {
                            if line.chunks[i - 1].is_whitespace || line.chunks[i].is_whitespace {
                                num_whitespace_gaps += 1;
                            }
                        }

                        let gap_width = remaining_width / num_whitespace_gaps as f32;
                        for i in 1..line.chunks.len() {
                            if line.chunks[i - 1].is_whitespace || line.chunks[i].is_whitespace {
                                line.chunks[i].left_adjust = -gap_width;
                            }
                        }
                    }
                }
            }

            lines
        }
    }
}

fn chunk_inline<'a>(fonts: &BTreeMap<&str, &'a Font>, inline: &Inline<'a>) -> Vec<Chunk<'a>> {
    let font = fonts.get(inline.style.font).unwrap();
    let font_scale = inline.style.font_size / font.face.units_per_em() as f32;

    let text_metrics = font.metrics() * inline.style.font_size;
    let style = inline.style;

    let mut chunks = Vec::new();
    let mut current_chunk_start = 0;
    let mut current_chunk_width = 0.0;
    for (i, c) in inline.text.char_indices() {
        let glyph_id = font.face.glyph_index(c).unwrap();
        let width = font.face.glyph_hor_advance(glyph_id).unwrap() as f32 * font_scale;

        if c.is_whitespace() {
            let next_i = i + c.len_utf8();

            if current_chunk_start < i {
                let prev_chunk = Chunk {
                    text: &inline.text[current_chunk_start..i],
                    style,
                    text_metrics,
                    width: current_chunk_width,
                    is_whitespace: false,
                    left_adjust: 0.0,
                };
                chunks.push(prev_chunk);
            }

            let chunk = Chunk {
                text: &inline.text[i..next_i],
                style,
                text_metrics,
                width,
                is_whitespace: true,
                left_adjust: 0.0,
            };
            chunks.push(chunk);

            current_chunk_start = next_i;
            current_chunk_width = 0.0;
        } else {
            current_chunk_width += width;
        }
    }

    if current_chunk_start < inline.text.len() {
        let final_chunk = Chunk {
            text: &inline.text[current_chunk_start..],
            style,
            text_metrics,
            width: current_chunk_width,
            is_whitespace: false,
            left_adjust: 0.0,
        };
        chunks.push(final_chunk);
    }

    chunks
}
