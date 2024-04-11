use std::collections::BTreeMap;

use crate::{
    char_map::CharMap,
    document::Document,
    font::Font,
    pdf::{page::PageBuilder, PDFBuilder},
    text_layout::layout_document,
};

pub mod char_map;
pub mod document;
pub mod equation;
pub mod font;
pub mod pdf;
pub mod text_layout;

pub fn generate_pdf(document: Document) -> String {
    let pages = layout_document(&document).unwrap();

    let char_map = CharMap::from_document(&document);
    let new_font_buffers = document
        .fonts
        .iter()
        .map(|(&name, font)| (name, font.subset(&char_map)))
        .collect::<BTreeMap<&str, Vec<u8>>>();
    let new_fonts = new_font_buffers
        .iter()
        .map(|(&name, buffer)| (name, Font::new(buffer).unwrap()))
        .collect::<BTreeMap<&str, Font>>();

    let mut pdf_builder = PDFBuilder::new();
    for page in pages {
        let mut builder = PageBuilder::new();
        builder.text(&page.lines, &char_map).unwrap();
        let page = builder.build();
        pdf_builder.page(&page).unwrap();
    }
    pdf_builder.catalog(&new_fonts, &char_map).unwrap();
    let content = pdf_builder.build().unwrap();
    content
}
