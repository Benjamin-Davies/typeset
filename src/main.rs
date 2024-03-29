use std::fs;

use typeset::{
    font::Font,
    pdf::{page::PageBuilder, PDFBuilder},
};

fn main() {
    let font = Font::default();
    let text = include_str!("../examples/lorem_ipsum.txt");

    let page = PageBuilder::new().paragraph(&font, 12.0, text).build();
    let content = PDFBuilder::new().single_page(&page).build();

    fs::write("target/output.pdf", content).unwrap();
}
