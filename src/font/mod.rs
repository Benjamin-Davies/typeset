use std::ops::Mul;

use thiserror::Error;
use ttf_parser::{name_id, Face};

mod generate;

pub struct Font<'a> {
    pub data: &'a [u8],
    pub face: Face<'a>,
    pub ps_name: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse face: {0}")]
    FaceParsing(#[from] ttf_parser::FaceParsingError),
    #[error("missing post script name")]
    MissingPostScriptName,
    #[error("non-unicode string")]
    NonUnicodeString,
}

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/noto-serif/NotoSerif-Regular.ttf"
));

impl Default for Font<'static> {
    fn default() -> Self {
        let font = Self::new(FONT_DATA).unwrap();
        assert_eq!(font.ps_name, "NotoSerif");
        font
    }
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Error> {
        let face = Face::parse(data, 0)?;

        let ps_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == name_id::POST_SCRIPT_NAME)
            .ok_or(Error::MissingPostScriptName)?
            .to_string()
            .ok_or(Error::NonUnicodeString)?;

        Ok(Self {
            data,
            face,
            ps_name,
        })
    }

    /// Converts from font units to thousanths of an em.
    pub fn to_milli_em(&self, units: i16) -> i32 {
        1000 * units as i32 / self.face.units_per_em() as i32
    }

    pub fn metrics(&self) -> TextMetrics {
        let scale = 1.0 / self.face.units_per_em() as f32;
        TextMetrics {
            ascent: self.face.ascender() as f32 * scale,
            descent: self.face.descender() as f32 * scale,
            line_gap: self.face.line_gap() as f32 * scale,
        }
    }
}

impl TextMetrics {
    pub fn max(&self, other: Self) -> Self {
        Self {
            ascent: self.ascent.max(other.ascent), // Choose the uppermost ascent.
            descent: self.descent.min(other.descent), // Choose the lowermost descent.
            line_gap: self.line_gap.max(other.line_gap), // Choose the largest line gap.
        }
    }

    pub fn line_height(&self) -> f32 {
        self.line_gap + self.ascent - self.descent
    }
}

impl Mul<f32> for TextMetrics {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            ascent: self.ascent * rhs,
            descent: self.descent * rhs,
            line_gap: self.line_gap * rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::font::{Font, TextMetrics};

    #[allow(clippy::format_collect)] // We don't care about small optimisations in tests.
    fn sha256_as_hex(data: &[u8]) -> String {
        use ring::digest;

        let hash = digest::digest(&digest::SHA256, data);
        hash.as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    #[test]
    fn test_font_hash() {
        let font = Font::default();

        let hash = sha256_as_hex(font.data);
        assert_eq!(
            hash,
            "01d6ee04157e31417f79c2a1beb9a578e0ebcf3ac2f9bc34a7d8d8d973e3081f",
        );
    }

    #[test]
    fn test_font_details() {
        let font = Font::default();

        assert_eq!(font.ps_name, "NotoSerif");

        let metrics = font.metrics();
        assert_eq!(metrics.ascent, 1.0688477);
        assert_eq!(metrics.descent, -0.29296875);
        assert_eq!(metrics.line_gap, 0.0);
    }

    #[test]
    fn test_to_milli_em() {
        let font = Font::default();

        // The default font's `units_per_em` is 2048.
        assert_eq!(font.to_milli_em(0), 0);
        assert_eq!(font.to_milli_em(256), 125);
        assert_eq!(font.to_milli_em(-512), -250);
    }

    #[test]
    fn test_metrics_max() {
        let metrics1 = TextMetrics {
            ascent: 1.2,
            descent: -0.2,
            line_gap: 0.0,
        };
        let metrics2 = TextMetrics {
            ascent: 1.0,
            descent: -0.5,
            line_gap: 0.1,
        };

        let max_metrics = metrics1.max(metrics2);
        assert_eq!(max_metrics.ascent, 1.2);
        assert_eq!(max_metrics.descent, -0.5);
        assert_eq!(max_metrics.line_gap, 0.1);
    }

    #[test]
    fn test_line_height() {
        let metrics = TextMetrics {
            ascent: 1.0688477,
            descent: -0.29296875,
            line_gap: 0.0,
        };
        assert_eq!(metrics.line_height(), 1.3618164);
    }

    #[test]
    fn test_metrics_mul() {
        let metrics = TextMetrics {
            ascent: 1.0688477,
            descent: -0.29296875,
            line_gap: 0.0,
        };

        let scaled_metrics = metrics * 12.0;
        assert_eq!(scaled_metrics.ascent, 12.826172);
        assert_eq!(scaled_metrics.descent, -3.515625);
        assert_eq!(scaled_metrics.line_gap, 0.0);
    }
}
