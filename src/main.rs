use std::fs;

use typeset::pdf::{page::PageBuilder, PDFBuilder};

fn main() {
    let page = PageBuilder::new().paragraph("Hello, world!").build();
    let content = PDFBuilder::new().single_page(&page).build();
    fs::write("target/output.pdf", content).unwrap();
}
