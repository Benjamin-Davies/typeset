use ttf_parser::{head::IndexToLocationFormat, GlyphId, LazyArray16, RawFace, TableRecord, Tag};

use crate::char_map::CharMap;

use super::Font;

const FIXED_HEADER_LEN: usize = 12;
const TABLE_RECORD_LEN: usize = 16;

impl Font<'_> {
    fn glyph_data(&self, glyph_id: GlyphId) -> &[u8] {
        let raw_face = *self.face.raw_face();
        let loca_data = raw_face.table(Tag::from_bytes(b"loca")).unwrap_or_default();
        let glyf_data = raw_face.table(Tag::from_bytes(b"glyf")).unwrap_or_default();

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
        let loca_data = raw_face.table(Tag::from_bytes(b"loca")).unwrap();
        let loca = LazyArray16::<u32>::new(&loca_data);
        let mut new_loca = vec![u32::MAX; loca.len() as usize];

        // Map from new glyph ID (index) to old glyph ID (value)
        let mut glyph_map = char_map
            .mappings
            .iter()
            .map(|&c| self.face.glyph_index(c).unwrap_or_default())
            .collect::<Vec<_>>();

        // BFS to check we have all glyph dependencies
        let mut i = 0;
        while i < glyph_map.len() {
            let glyph_id = glyph_map[i];
            let glyph_data = self.glyph_data(glyph_id);
            if glyph_data.is_empty() {
                i += 1;
                continue;
            }

            let num_contours = i16::from_be_bytes([glyph_data[0], glyph_data[1]]);
            if num_contours == -1 {
                // Compound glyph
                let mut j = 10;
                while j < glyph_data.len() {
                    let flags = u16::from_be_bytes([glyph_data[j], glyph_data[j + 1]]);
                    let component_glyph_id =
                        GlyphId(u16::from_be_bytes([glyph_data[j + 2], glyph_data[j + 3]]));

                    if !glyph_map.contains(&component_glyph_id) {
                        glyph_map.push(component_glyph_id);
                    }

                    // The size of the glyph table depends on the flags
                    j += 4;
                    if flags & 0x0001 != 0 {
                        j += 4;
                    } else {
                        j += 2;
                    }
                    if flags & 0x0008 != 0 {
                        j += 2;
                    } else if flags & 0x0040 != 0 {
                        j += 4;
                    } else if flags & 0x0080 != 0 {
                        j += 8;
                    }

                    // More components flag == 0
                    if flags & 0x0020 == 0 {
                        break;
                    }
                }
            }

            i += 1;
        }

        let mut contents = Vec::new();
        copy_header(&mut contents, raw_face);
        for (i, mut table_record) in raw_face.table_records.into_iter().enumerate() {
            let table_data = raw_face.table(table_record.tag).unwrap();

            pad_to_multiple_of(&mut contents, 4);

            let (offset, len) = match &table_record.tag.to_bytes() {
                b"cmap" => write_cmap(&mut contents, &glyph_map),
                b"glyf" => write_glyf(&mut contents, &glyph_map, loca, table_data, &mut new_loca),
                b"loca" => write_loca(&mut contents, &new_loca),
                _ => copy_table(&mut contents, &raw_face.data, table_record),
            };

            // Most modern rasterizers ignore the checksums, so we don't bother updating them.
            table_record.offset = offset;
            table_record.length = len;
            write_table_record(&mut contents, i, table_record);
        }

        contents
    }
}

fn write_cmap(contents: &mut Vec<u8>, glyph_map: &[GlyphId]) -> (u32, u32) {
    let cmap_offset = contents.len() as u32;

    // cmap Header
    contents.push_u16(0); // version
    contents.push_u16(1); // num tables

    // Encoding Record
    contents.push_u16(0); // platform ID = Unicode
    contents.push_u16(3); // encoding ID = 2.0+, BMP only
    contents.push_u32(12); // subtable offset

    // Subtable Header
    let subtable_start_global_offset = contents.len();
    contents.push_u16(4); // format = 4 (Segment mapping to delta values)
    let subtable_len_global_offset = contents.len();
    contents.push_u16(0); // length (backfilled later)
    contents.push_u16(0); // language (not used)
    let seg_count = glyph_map.len() as u16 + 1; // Add one at the end for the last end code
    let seg_count_log_2 = 15 - seg_count.leading_zeros();
    contents.push_u16(2 * seg_count); // segment count * 2
    contents.push_u16(2 << seg_count_log_2); // search range
    contents.push_u16(seg_count_log_2 as u16); // entry selector
    contents.push_u16(2 * seg_count - 2 << seg_count_log_2); // range shift

    // end code
    for (i, _glyph_id) in glyph_map.iter().enumerate() {
        contents.push_u16(i as u16);
    }
    contents.push_u16(u16::MAX); // last end code

    // reserved padding
    contents.push_u16(0);

    // start code
    for (i, _glyph_id) in glyph_map.iter().enumerate() {
        contents.push_u16(i as u16);
    }
    contents.push_u16(u16::MAX); // last start code

    // ID delta
    for (i, &glyph_id) in glyph_map.iter().enumerate() {
        let delta = glyph_id.0 as i16 - i as i16;
        contents.push_u16(delta as u16);
    }
    contents.push_u16(0); // last ID delta

    // ID range offset
    for _ in glyph_map {
        contents.push_u16(0);
    }
    contents.push_u16(0); // last ID range offset

    // backfill the length
    let subtable_len = contents.len() as u32 - subtable_start_global_offset as u32;
    contents[subtable_len_global_offset..subtable_len_global_offset + 4]
        .copy_from_slice(&subtable_len.to_be_bytes());

    let cmap_len = contents.len() as u32 - cmap_offset;
    (cmap_offset, cmap_len)
}

fn write_glyf(
    contents: &mut Vec<u8>,
    glyph_map: &[GlyphId],
    loca: LazyArray16<u32>,
    table_data: &[u8],
    new_loca: &mut [u32],
) -> (u32, u32) {
    let glyf_offset = contents.len() as u32;
    let mut glyf_len = 0u32;

    for &glyph_id in glyph_map {
        let offset = loca.get(glyph_id.0).unwrap_or_default();
        let next_offset = loca.get(glyph_id.0 + 1).unwrap_or_default();
        let len = next_offset - offset;
        if len == 0 {
            continue;
        }

        let new_locus = glyf_len;
        contents.extend_from_slice(&table_data[offset as usize..next_offset as usize]);
        glyf_len += len;

        new_loca[glyph_id.0 as usize] = new_locus;
    }

    for locus in new_loca {
        if *locus == u32::MAX {
            *locus = glyf_len;
        }
    }

    (glyf_offset, glyf_len)
}

fn write_loca(contents: &mut Vec<u8>, new_loca: &[u32]) -> (u32, u32) {
    let loca_offset = contents.len() as u32;
    let loca_len = new_loca.len() as u32 * 4;

    // The loca table is always written after the glyf table.
    for &offset in new_loca {
        contents.push_u32(offset);
    }

    (loca_offset, loca_len)
}

fn copy_header(contents: &mut Vec<u8>, raw_face: RawFace) {
    // Copy the header from the original font
    let len = FIXED_HEADER_LEN + TABLE_RECORD_LEN * raw_face.table_records.len() as usize;
    contents.extend_from_slice(&raw_face.data[..len]);
}

fn copy_table(contents: &mut Vec<u8>, data: &[u8], table_record: TableRecord) -> (u32, u32) {
    let offset = table_record.offset as usize;
    let len = table_record.length as usize;

    let new_offset = contents.len() as u32;
    contents.extend(&data[offset..offset + len]);
    (new_offset, len as u32)
}

fn write_table_record(
    contents: &mut [u8], // We don't extend the contents here
    index: usize,
    table_record: TableRecord,
) {
    let offset = FIXED_HEADER_LEN + TABLE_RECORD_LEN * index;
    contents[offset..offset + 4].copy_from_slice(&table_record.tag.0.to_be_bytes());
    contents[offset + 4..offset + 8].copy_from_slice(&table_record.check_sum.to_be_bytes());
    contents[offset + 8..offset + 12].copy_from_slice(&table_record.offset.to_be_bytes());
    contents[offset + 12..offset + 16].copy_from_slice(&table_record.length.to_be_bytes());
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
