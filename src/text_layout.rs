use crate::font::Font;

pub fn split_line<'a>(font: &Font, font_size: f32, s: &'a str, width: f32) -> (&'a str, &'a str) {
    let width = (width / font_size * font.face.units_per_em() as f32) as u32;

    let mut last_space = s.len();
    let mut x = 0;
    for (i, c) in s.char_indices() {
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
            return s.split_at(last_space + 1);
        }
    }

    (s, "")
}
