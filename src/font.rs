use std::ops::{Mul, RangeInclusive};

use ttf_parser::{name_id, Face};

pub struct Font<'a> {
    pub data: &'a [u8],
    pub face: Face<'a>,
    pub ps_name: String,
    pub char_range: RangeInclusive<char>,
    pub widths: Vec<u32>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/noto-serif/NotoSerif-Regular.ttf"
));

impl Default for Font<'static> {
    fn default() -> Self {
        let font = Self::new(FONT_DATA);
        assert_eq!(font.ps_name, "NotoSerif");
        font
    }
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let face = Face::parse(data, 0).unwrap();

        let ps_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == name_id::POST_SCRIPT_NAME)
            .unwrap()
            .to_string()
            .unwrap();

        let char_range = '\x00'..='\x7F';
        let units_per_em = face.units_per_em() as u32;
        let widths = char_range
            .clone()
            .map(|c| {
                let glyph_id = face.glyph_index(c)?;
                let width = face.glyph_hor_advance(glyph_id)?;
                // Convert from font units to thousandths of an em.
                Some(1000 * width as u32 / units_per_em)
            })
            .map(|w| w.unwrap_or(0))
            .collect::<Vec<_>>();

        Self {
            data,
            face,
            ps_name,
            char_range,
            widths,
        }
    }

    /// Converts from font units to thousanths of an em.
    pub fn to_milli_em(&self, units: i16) -> i32 {
        1000 * units as i32 / self.face.units_per_em() as i32
    }

    pub fn metrics(&self) -> FontMetrics {
        let scale = 1.0 / self.face.units_per_em() as f32;
        FontMetrics {
            ascent: self.face.ascender() as f32 * scale,
            descent: self.face.descender() as f32 * scale,
            line_gap: self.face.line_gap() as f32 * scale,
        }
    }
}

impl FontMetrics {
    pub fn max(&self, other: Self) -> Self {
        Self {
            ascent: self.ascent.max(other.ascent), // Choose the uppermost ascent.
            descent: self.descent.min(other.descent), // Choose the lowermost descent.
            line_gap: self.line_gap.max(other.line_gap), // Choose the largest line gap.
        }
    }
}

impl Mul<f32> for FontMetrics {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            ascent: self.ascent * rhs,
            descent: self.descent * rhs,
            line_gap: self.line_gap * rhs,
        }
    }
}
