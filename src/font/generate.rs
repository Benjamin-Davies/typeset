use std::collections::BTreeMap;

use ttf_parser::{head::IndexToLocationFormat, Face, GlyphId, LazyArray16, RawFace, Tag};

use crate::char_map::CharMap;

use super::Font;

const CMAP: Tag = Tag::from_bytes(b"cmap");
const GLYF: Tag = Tag::from_bytes(b"glyf");
const HEAD: Tag = Tag::from_bytes(b"head");
const HHEA: Tag = Tag::from_bytes(b"hhea");
const HMTX: Tag = Tag::from_bytes(b"hmtx");
const LOCA: Tag = Tag::from_bytes(b"loca");
const MAXP: Tag = Tag::from_bytes(b"maxp");
const NAME: Tag = Tag::from_bytes(b"name");
const POST: Tag = Tag::from_bytes(b"post");

const FIXED_HEADER_LEN: usize = 12;
const TABLE_RECORD_LEN: usize = 16;

impl Font<'_> {
    fn glyph_data(&self, glyph_id: GlyphId) -> &[u8] {
        let raw_face = *self.face.raw_face();
        let loca_data = raw_face.table(LOCA).unwrap_or_default();
        let glyf_data = raw_face.table(GLYF).unwrap_or_default();

        let loca = LazyArray16::<u32>::new(&loca_data);
        let offset = loca.get(glyph_id.0).unwrap_or_default() as usize;
        let next_offset = loca.get(glyph_id.0 + 1).unwrap_or(glyf_data.len() as u32) as usize;

        &glyf_data[offset..next_offset]
    }

    pub fn subset(&self, char_map: &CharMap) -> Vec<u8> {
        assert!(self.face.is_subsetting_allowed());
        assert_eq!(
            self.face.tables().head.index_to_location_format,
            IndexToLocationFormat::Long
        );

        let raw_face = *self.face.raw_face();

        // Map from new glyph ID (index) to old glyph ID (value)
        let mut glyph_map = char_map
            .mappings
            .iter()
            .map(|&c| self.face.glyph_index(c).unwrap_or_default())
            .collect::<Vec<_>>();
        glyph_map[0] = GlyphId(0); // The first glyph must be the missing glyph
        collect_glyph_dependencies(self, &mut glyph_map);

        // Use a BTreeMap to keep the tables sorted by tag
        let mut tables = BTreeMap::new();
        tables.insert(CMAP, generate_cmap());
        let mut loca = Vec::new();
        tables.insert(GLYF, generate_glyf(self, &glyph_map, &mut loca));
        tables.insert(LOCA, generate_loca(&loca));
        tables.insert(HMTX, generate_hmtx(&self.face, &glyph_map));
        tables.insert(HHEA, generate_hhea(&raw_face, glyph_map.len()));

        // Copy the rest of the required tables verbatim
        for tag in [HEAD, MAXP, NAME, POST] {
            tables.insert(tag, raw_face.table(tag).unwrap_or_default().to_owned());
        }

        let mut contents = Vec::new();

        // table directory
        contents.push_u32(0x00010000); // version
        contents.push_u16(tables.len() as u16); // num tables
        let (search_range, entry_selector, range_shift) = search_hints(tables.len() as u16);
        contents.push_u16(16 * search_range); // search range
        contents.push_u16(entry_selector); // entry selector
        contents.push_u16(16 * range_shift); // range shift
        for &tag in tables.keys() {
            // table records
            contents.push_u32(tag.0); // tag
            contents.push_u32(0); // check sum (ignored)
            contents.push_u32(0); // offset (backfilled later)
            contents.push_u32(0); // length (backfilled later)
        }

        // tables
        for (i, (tag, mut table)) in tables.into_iter().enumerate() {
            match table {
                _ if tag == MAXP => {
                    // num glyphs
                    table[4..6].copy_from_slice(&(glyph_map.len() as u16).to_be_bytes());
                }
                _ => {}
            }

            pad_to_multiple_of(&mut contents, 8);

            let offset = contents.len() as u32;
            let len = table.len() as u32;
            contents.extend_from_slice(&table);

            // backfill the offset and length
            let table_record_offset = FIXED_HEADER_LEN + TABLE_RECORD_LEN * i;
            contents[table_record_offset + 8..table_record_offset + 12]
                .copy_from_slice(&offset.to_be_bytes());
            contents[table_record_offset + 12..table_record_offset + 16]
                .copy_from_slice(&len.to_be_bytes());
        }

        contents
    }
}

/// BFS to collect all the dependencies for composite glyphs.
fn collect_glyph_dependencies(font: &Font, glyph_map: &mut Vec<GlyphId>) {
    let mut i = 0;
    while i < glyph_map.len() {
        let glyph_id = glyph_map[i];
        let glyph_data = font.glyph_data(glyph_id);
        if glyph_data.is_empty() {
            i += 1;
            continue;
        }

        let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);
        if num_contours == -1 {
            let mut j = 10;
            while j < glyph_data.len() {
                let flags = u16::from_be_bytes([glyph_data[j], glyph_data[j + 1]]);
                let component_glyph_id =
                    GlyphId(u16::from_be_bytes([glyph_data[j + 2], glyph_data[j + 3]]));

                if !glyph_map.contains(&component_glyph_id) {
                    glyph_map.push(component_glyph_id);
                }

                j += component_glyph_table_len(flags);
                // More components flag == 0
                if flags & 0x0020 == 0 {
                    break;
                }
            }
        }

        i += 1;
    }
}

fn component_glyph_table_len(flags: u16) -> usize {
    let mut len = 4;
    if flags & 0x0001 != 0 {
        len += 4;
    } else {
        len += 2;
    }
    if flags & 0x0008 != 0 {
        len += 2;
    } else if flags & 0x0040 != 0 {
        len += 4;
    } else if flags & 0x0080 != 0 {
        len += 8;
    }

    len
}

fn generate_cmap() -> Vec<u8> {
    let mut contents = Vec::new();

    // cmap Header
    contents.push_u16(0); // version
    contents.push_u16(1); // num tables

    // Encoding Record
    contents.push_u16(0); // platform ID = Unicode
    contents.push_u16(3); // encoding ID = 2.0+, BMP only
    contents.push_u32(12); // subtable offset

    // Subtable Header
    contents.push_u16(4); // format = 4 (Segment mapping to delta values)
    contents.push_u16(24); // length
    contents.push_u16(0); // language (not used)
    contents.push_u16(2); // segment count * 2
    let (search_range, entry_selector, range_shift) = search_hints(1);
    contents.push_u16(2 * search_range); // search range
    contents.push_u16(entry_selector); // entry selector
    contents.push_u16(2 * range_shift); // range shift
    contents.push_u16(u16::MAX); // end code
    contents.push_u16(0); // reserved padding
    contents.push_u16(0); // start code
    contents.push_u16(0); // ID delta
    contents.push_u16(0); // ID range offset

    contents
}

fn generate_glyf(font: &Font, glyph_map: &[GlyphId], loca: &mut Vec<u32>) -> Vec<u8> {
    const GLYF_LEN_MARKER: u32 = u32::MAX;

    let mut contents = Vec::new();

    for &glyph_id in glyph_map {
        let glyph_data = font.glyph_data(glyph_id);
        if glyph_data.is_empty() {
            loca.push(GLYF_LEN_MARKER);
            continue;
        }

        pad_to_multiple_of(&mut contents, 8);

        let start = contents.len();
        contents.extend_from_slice(glyph_data);

        map_glyph_components(&mut contents[start..], glyph_map);

        loca.push(start as u32);
    }
    loca.push(GLYF_LEN_MARKER);

    let glyf_len = contents.len() as u32;
    for locus in loca {
        if *locus == GLYF_LEN_MARKER {
            *locus = glyf_len;
        }
    }

    contents
}

/// Updates the component glyph references to the new glyph indices if this is a composite glyph.
fn map_glyph_components(glyph_data: &mut [u8], glyph_map: &[GlyphId]) {
    let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);
    if num_contours == -1 {
        let mut j = 10;
        while j < glyph_data.len() {
            let flags = u16::from_be_bytes([glyph_data[j], glyph_data[j + 1]]);
            let component_glyph_id =
                GlyphId(u16::from_be_bytes([glyph_data[j + 2], glyph_data[j + 3]]));

            let component_glyph_index = glyph_map
                .iter()
                .position(|&id| id == component_glyph_id)
                .unwrap_or_default();
            glyph_data[j + 2..j + 4].copy_from_slice(&(component_glyph_index as u16).to_be_bytes());

            j += component_glyph_table_len(flags);
            // More components flag == 0
            if flags & 0x0020 == 0 {
                break;
            }
        }
    }
}

fn generate_loca(loca: &[u32]) -> Vec<u8> {
    let mut contents = Vec::new();

    for &offset in loca {
        contents.push_u32(offset);
    }

    contents
}

fn generate_hmtx(face: &Face, glyph_map: &[GlyphId]) -> Vec<u8> {
    let mut contents = Vec::new();

    for &glyph_id in glyph_map {
        let advance_width = face.glyph_hor_advance(glyph_id).unwrap_or_default();
        let left_side_bearing = face.glyph_hor_side_bearing(glyph_id).unwrap_or_default();

        contents.push_u16(advance_width);
        contents.push_u16(left_side_bearing as u16);
    }

    contents
}

fn generate_hhea(raw_face: &RawFace, num_glyphs: usize) -> Vec<u8> {
    let mut contents = raw_face.table(HHEA).unwrap().to_owned();

    // number of H-metrics
    contents[34..36].copy_from_slice(&(num_glyphs as u16).to_be_bytes());

    contents
}

fn search_hints(len: u16) -> (u16, u16, u16) {
    let mut search_range = 1;
    let mut entry_selector = 0;
    let range_shift;

    while search_range as u32 * 2 <= len as u32 {
        search_range *= 2;
        entry_selector += 1;
    }
    range_shift = len - search_range;

    (search_range, entry_selector, range_shift)
}

fn pad_to_multiple_of(contents: &mut Vec<u8>, alignment: usize) {
    while contents.len() % alignment != 0 {
        contents.push(0);
    }
}

trait VecExt: Extend<u8> {
    fn push_u16(&mut self, value: u16) {
        self.extend(value.to_be_bytes());
    }

    fn push_u32(&mut self, value: u32) {
        self.extend(value.to_be_bytes());
    }
}

impl VecExt for Vec<u8> {}
