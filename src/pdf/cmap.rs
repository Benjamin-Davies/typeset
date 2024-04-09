use std::fmt::{self, Write};

use crate::{char_map::CharMap, font::Font};

use super::{PDFBuilder, Ref};

pub struct MappedStr<'a>(pub &'a str, pub &'a CharMap);

impl fmt::Display for MappedStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<")?;

        let Self(s, char_map) = self;
        for c in s.chars() {
            let code = char_map.get(c).ok_or_else(|| {
                eprintln!("Character not found in char map: {c:?}");
                fmt::Error
            })?;
            write!(f, "{:02x}", code)?;
        }

        write!(f, ">")?;
        Ok(())
    }
}

impl PDFBuilder {
    pub(super) fn cmap(&mut self, font: &Font, char_map: &CharMap) -> Result<Ref, fmt::Error> {
        let mut cmap = String::new();
        write_cmap(&mut cmap, font, char_map)?;

        self.stream_object(&cmap)
    }
}

fn write_cmap(s: &mut String, font: &Font, char_map: &CharMap) -> Result<(), fmt::Error> {
    // Copied from LibreOffice output
    writeln!(s, "/CIDInit /ProcSet findresource begin")?;
    writeln!(s, "12 dict begin")?;
    writeln!(s, "begincmap")?;
    writeln!(s, "/CIDSystemInfo<<")?;
    writeln!(s, "/Registry (Adobe)")?;
    writeln!(s, "/Ordering (UCS)")?;
    writeln!(s, "/Supplement 0")?;
    writeln!(s, ">> def")?;
    writeln!(s, "/CMapName /Adobe-Identity-UCS def")?;
    writeln!(s, "/CMapType 2 def")?;
    writeln!(s, "1 begincodespacerange")?;
    writeln!(s, "<00> <FF>")?;
    writeln!(s, "endcodespacerange")?;

    writeln!(s, "{} beginbfchar", char_map.mappings.len())?;
    for (i, &c) in char_map.mappings.iter().enumerate() {
        let glyph_id = font.face.glyph_index(c).unwrap_or_default();
        writeln!(s, "<{:02x}> <{:04x}>", i, c as u32)?;
    }
    writeln!(s, "endbfchar")?;

    writeln!(s, "endcmap")?;
    writeln!(s, "CMapName currentdict /CMap defineresource pop")?;
    writeln!(s, "end")?;
    writeln!(s, "end")?;

    Ok(())
}
