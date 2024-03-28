use std::fs;

use typeset::pdf::PDFBuilder;

fn main() {
    let s = "Hello, World!";
    let face = typeset::font::read_font();
    let xs = typeset::text_layout::compute_x_positions(&face, 12.0, s);

    let content = PDFBuilder::new().single_page().build();
    fs::write("target/output.pdf", content).unwrap();
}
