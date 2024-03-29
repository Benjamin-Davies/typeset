use crate::font::Font;

pub fn layout_paragraphs<'a>(
    font: &Font,
    font_size: f32,
    s: &'a str,
    width: f32,
) -> Vec<Vec<&'a str>> {
    let width = (width / font_size * font.face.units_per_em() as f32) as u32;

    let mut paragraphs = Vec::new();
    for line in s.lines() {
        let mut rest = line;
        let mut paragraph = Vec::new();
        while !rest.is_empty() {
            let mut last_space = rest.len();
            let mut x = 0;
            for (i, c) in rest.char_indices() {
                let Some(glyph_id) = font.face.glyph_index(c) else {
                    continue;
                };
                let Some(advance) = font.face.glyph_hor_advance(glyph_id) else {
                    continue;
                };
                x += advance as u32;
                if c == ' ' {
                    last_space = i;
                }
                if x > width {
                    break;
                }
            }

            if x <= width {
                paragraph.push(rest);
                break;
            }

            paragraph.push(&rest[..last_space]);
            rest = &rest[last_space..].trim_start();
        }

        paragraphs.push(paragraph);
    }

    paragraphs
}
