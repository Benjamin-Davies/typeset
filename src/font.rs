use ttf_parser::Face;

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/noto-serif/NotoSerif-Regular.ttf"
));

pub fn read_font() {
    let face = Face::parse(FONT_DATA, 0).unwrap();
    dbg!(face.number_of_glyphs());
    dbg!(face.names().into_iter().collect::<Vec<_>>());
}
