use ttf_parser::Face;

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/noto-serif/NotoSerif-Regular.ttf"
));

pub fn read_font() -> Face<'static> {
    Face::parse(FONT_DATA, 0).unwrap()
}
