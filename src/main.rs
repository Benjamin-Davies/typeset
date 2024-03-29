use std::fs;

use typeset::{
    font::read_font,
    pdf::{page::PageBuilder, PDFBuilder},
};

fn main() {
    let face = read_font();

    let page = PageBuilder::new()
        .paragraph(&face, 12.0, "Hello, world! Typesetting is fun.")
        .build();
    let content = PDFBuilder::new().single_page(&page).build();

    fs::write("target/output.pdf", content).unwrap();
}
