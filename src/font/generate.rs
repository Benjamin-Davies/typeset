use ttf_parser::{Face, RawFace, TableRecord};

use crate::char_map::CharMap;

use super::Font;

const FIXED_HEADER_LEN: usize = 12;
const TABLE_RECORD_LEN: usize = 16;

impl Font<'_> {
    pub fn with_cmap(&self, char_map: &CharMap) -> Vec<u8> {
        let raw_face = *self.face.raw_face();

        let mut contents = Vec::new();
        copy_header(&mut contents, raw_face);
        for (i, mut table_record) in raw_face.table_records.into_iter().enumerate() {
            pad_to_multiple_of(&mut contents, 4);

            let (offset, len) = match &table_record.tag.to_bytes() {
                b"cmap" => write_cmap(&mut contents, char_map, &self.face),
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

fn write_cmap(contents: &mut Vec<u8>, char_map: &CharMap, old_face: &Face) -> (u32, u32) {
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
    let seg_count = char_map.mappings.len() as u16 + 1; // Add one at the end for the last end code
    let seg_count_log_2 = 15 - seg_count.leading_zeros();
    contents.push_u16(2 * seg_count); // segment count * 2
    contents.push_u16(2 << seg_count_log_2); // search range
    contents.push_u16(seg_count_log_2 as u16); // entry selector
    contents.push_u16(2 * seg_count - 2 << seg_count_log_2); // range shift

    // end code
    for (i, _c) in char_map.mappings.iter().enumerate() {
        contents.push_u16(i as u16);
    }
    contents.push_u16(u16::MAX); // last end code

    // reserved padding
    contents.push_u16(0);

    // start code
    for (i, _c) in char_map.mappings.iter().enumerate() {
        contents.push_u16(i as u16);
    }
    contents.push_u16(u16::MAX); // last start code

    // ID delta
    for (i, &c) in char_map.mappings.iter().enumerate() {
        let glyph_id = old_face.glyph_index(c).unwrap_or_default();
        let delta = glyph_id.0 as i16 - i as i16;
        contents.push_u16(delta as u16);
    }
    contents.push_u16(0); // last ID delta

    // ID range offset
    for _ in char_map.mappings.iter() {
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
