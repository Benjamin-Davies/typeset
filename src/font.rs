use ttf_parser::Face;

pub const FONT_PS_NAME: &str = "NotoSerif";
pub const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/noto-serif/NotoSerif-Regular.ttf"
));

pub fn read_font() -> Face<'static> {
    Face::parse(FONT_DATA, 0).unwrap()
}
