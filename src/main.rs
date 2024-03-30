use std::{collections::BTreeMap, fs};

use glam::vec2;
use typeset::{
    document::{Block, Document, Inline, Style, TextAlign, TextBlock},
    font::Font,
    pdf::{
        page::{PageBuilder, PAGE_HEIGHT, PAGE_WIDTH},
        PDFBuilder,
    },
    text_layout::layout_document,
};

fn main() {
    let font = Font::default();
    let bold_font = Font::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/noto-serif/NotoSerif-Bold.ttf"
    )));
    let italic_font = Font::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/noto-serif/NotoSerif-Italic.ttf"
    )));
    let text = include_str!("../examples/lorem_ipsum.txt");

    let style = Style {
        font: &font.ps_name,
        font_size: 12.0,
    };
    let mut blocks = vec![
        Block::Text(TextBlock {
            inlines: vec![Inline {
                style: Style {
                    font_size: 2.0 * style.font_size,
                    ..style
                },
                text: "Hello, World!",
            }],
            align: TextAlign::Left,
        }),
        Block::Text(TextBlock {
            inlines: vec![
                Inline {
                    style,
                    text: "Regular, ",
                },
                Inline {
                    style: Style {
                        font: &bold_font.ps_name,
                        ..style
                    },
                    text: "bold, ",
                },
                Inline {
                    style: Style {
                        font: &italic_font.ps_name,
                        ..style
                    },
                    text: "or italic?",
                },
            ],
            align: TextAlign::Left,
        }),
    ];
    for (line, align) in text.lines().zip([
        TextAlign::Right,
        TextAlign::Left,
        TextAlign::Center,
        TextAlign::Right,
        TextAlign::Justify,
    ]) {
        blocks.push(Block::Text(TextBlock {
            inlines: vec![Inline { style, text: line }],
            align,
        }));
    }

    let mut fonts = BTreeMap::new();
    fonts.insert(&*font.ps_name, &font);
    fonts.insert(&*bold_font.ps_name, &bold_font);
    fonts.insert(&*italic_font.ps_name, &italic_font);

    let document = Document {
        blocks,
        fonts,
        page_size: vec2(PAGE_WIDTH, PAGE_HEIGHT),
        margin: 72.0,
    };

    let lines = layout_document(&document);

    let page = PageBuilder::new().text(&lines).build();
    let content = PDFBuilder::new()
        .single_page(&document.fonts, &page)
        .build();

    fs::write("target/output.pdf", content).unwrap();
}
