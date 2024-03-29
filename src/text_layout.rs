use crate::font::Font;

pub fn compute_x_positions(font: &Font, font_size: f32, s: &str) -> Vec<(char, f32)> {
    let face = &font.face;
    let font_scale = font_size / face.units_per_em() as f32;

    let mut x = 0.0;
    let mut xs = Vec::new();
    for c in s.chars() {
        xs.push((c, x));

        let glyph_id = face.glyph_index(c).unwrap();
        x += face.glyph_hor_advance(glyph_id).unwrap() as f32 * font_scale;
    }

    xs
}
