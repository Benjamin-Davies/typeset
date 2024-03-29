use std::fs;

use typeset::{
    font::Font,
    pdf::{page::PageBuilder, PDFBuilder},
};

fn main() {
    let font = Font::default();

    let page = PageBuilder::new()
        .paragraph(
            &font,
            12.0,
            "Hello, world! Typesetting is fun. Tokenisation more so.",
        )
        .build();
    let content = PDFBuilder::new().single_page(&page).build();

    fs::write("target/output.pdf", content).unwrap();
}
