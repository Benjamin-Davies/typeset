fn main() {
    let s = "Hello, World!";
    let face = typeset::font::read_font();
    let xs = typeset::text_layout::compute_x_positions(&face, 12.0, s);
    dbg!(xs);
}
