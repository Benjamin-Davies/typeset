use std::collections::BTreeMap;

use glam::{vec2, Vec2};

use crate::{
    document::{Block, Document, Inline, Style},
    font::{Font, FontMetrics},
};

const PARAGRAPH_GAP: f32 = 12.0;

#[derive(Debug, Clone)]
pub struct Line<'a> {
    pub chunks: Vec<Chunk<'a>>,
    pub font_metrics: FontMetrics,
    pub text_total_width: f32,
    pub delta: Vec2,
}

#[derive(Debug, Clone)]
pub struct Chunk<'a> {
    pub text: &'a str,
    pub style: Style<'a>,
    pub font_metrics: FontMetrics,
    pub width: f32,
    pub is_whitespace: bool,
    pub left_adjust: f32,
}

pub fn layout_document<'a>(document: &Document<'a>) -> Vec<Line<'a>> {
    let target_width = document.page_size.x - 2.0 * document.margin;

    let mut lines = Vec::new();
    for block in &document.blocks {
        let mut block_lines = layout_block(&document.fonts, target_width, block);

        if let Some(first_line) = block_lines.first_mut() {
            first_line.delta.y -= PARAGRAPH_GAP;
        }

        lines.append(&mut block_lines);
    }

    // TODO: page breaks

    if let Some(first_line) = lines.first_mut() {
        first_line.delta.x = document.margin;
        first_line.delta.y =
            document.page_size.y - document.margin - first_line.font_metrics.ascent;
    }
    lines
}

fn layout_block<'a>(
    fonts: &BTreeMap<&str, &'a Font>,
    target_width: f32,
    block: &Block<'a>,
) -> Vec<Line<'a>> {
    match block {
        Block::Text(block) => {
            let chunks = block
                .inlines
                .iter()
                .flat_map(|inline| chunk_inline(fonts, inline))
                .collect::<Vec<_>>();

            let mut lines = Vec::<Line>::new();
            let mut line_start = 0;
            let mut possible_break = 0;
            let mut current_line_width = 0.0;
            let mut current_line_font_metrics = FontMetrics::default();
            for (i, chunk) in chunks.iter().enumerate() {
                if possible_break > line_start
                    && current_line_width - chunk.left_adjust + chunk.width > target_width
                {
                    let mut line_spacing =
                        current_line_font_metrics.line_gap + current_line_font_metrics.ascent;
                    if let Some(prev_line) = lines.last() {
                        line_spacing -= prev_line.font_metrics.descent;
                    }

                    let line = Line {
                        chunks: chunks[line_start..possible_break].to_vec(),
                        font_metrics: current_line_font_metrics,
                        text_total_width: current_line_width,
                        delta: vec2(0.0, -line_spacing),
                    };
                    lines.push(line);

                    line_start = possible_break;
                    current_line_width = chunk.width;
                    current_line_font_metrics = chunk.font_metrics;
                } else {
                    current_line_width += chunk.width;
                    current_line_font_metrics = current_line_font_metrics.max(chunk.font_metrics);

                    if chunk.is_whitespace && !chunks[i.saturating_sub(1)].is_whitespace {
                        possible_break = i;
                    }
                }
            }

            if line_start < chunks.len() {
                let mut line_spacing =
                    current_line_font_metrics.line_gap + current_line_font_metrics.ascent;
                if let Some(prev_line) = lines.last() {
                    line_spacing -= prev_line.font_metrics.descent;
                }

                let line = Line {
                    chunks: chunks[line_start..].to_vec(),
                    font_metrics: current_line_font_metrics,
                    text_total_width: current_line_width,
                    delta: vec2(0.0, -line_spacing),
                };
                lines.push(line);
            }

            // TODO: alignment

            lines
        }
    }
}

fn chunk_inline<'a>(fonts: &BTreeMap<&str, &'a Font>, inline: &Inline<'a>) -> Vec<Chunk<'a>> {
    let font = fonts.get(inline.style.font).unwrap();
    let font_scale = inline.style.font_size / font.face.units_per_em() as f32;

    let font_metrics = font.metrics() * inline.style.font_size;
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
                    font_metrics,
                    width: current_chunk_width,
                    is_whitespace: false,
                    left_adjust: 0.0,
                };
                chunks.push(prev_chunk);
            }

            let chunk = Chunk {
                text: &inline.text[i..next_i],
                style,
                font_metrics,
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
            font_metrics,
            width: current_chunk_width,
            is_whitespace: false,
            left_adjust: 0.0,
        };
        chunks.push(final_chunk);
    }

    chunks
}
