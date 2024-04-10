use std::{collections::BTreeMap, fs};

use glam::vec2;
use typeset::{
    document::{Block, Document, Inline, Style, TextAlign, TextBlock},
    font::Font,
    generate_pdf,
    pdf::page::{PAGE_HEIGHT, PAGE_WIDTH},
};

#[test]
fn test_example() {
    let font = Font::default();
    let bold_font = Font::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/noto-serif/NotoSerif-Bold.ttf"
    )))
    .unwrap();
    let italic_font = Font::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/noto-serif/NotoSerif-Italic.ttf"
    )))
    .unwrap();
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
        TextAlign::Justify,
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

    let content = generate_pdf(document);

    fs::create_dir_all("output").unwrap();
    fs::write("output/lorem_ipsum.pdf", &content).unwrap();

    assert_eq!(content, include_str!("../examples/lorem_ipsum.pdf"));
}

#[test]
fn test_greek() {
    let font = Font::default();
    let text = "Εν αρχη ην ο λογος, και ο λογος ην προς τον θεον, και θεος ην ο λογος.";
    let text2 = "Ἐν ἀρχῇ ἦν ὁ λόγος, καὶ ὁ λόγος ἦν πρὸς τὸν θεόν, καὶ θεὸς ἦν ὁ λόγος.";

    let style = Style {
        font: &font.ps_name,
        font_size: 12.0,
    };
    let blocks = vec![
        Block::Text(TextBlock {
            inlines: vec![Inline { style, text }],
            align: TextAlign::Left,
        }),
        Block::Text(TextBlock {
            inlines: vec![Inline { style, text: text2 }],
            align: TextAlign::Left,
        }),
    ];

    let mut fonts = BTreeMap::new();
    fonts.insert(&*font.ps_name, &font);

    let document = Document {
        blocks,
        fonts,
        page_size: vec2(PAGE_WIDTH, PAGE_HEIGHT),
        margin: 72.0,
    };

    let content = generate_pdf(document);

    fs::create_dir_all("output").unwrap();
    fs::write("output/greek.pdf", &content).unwrap();

    assert_eq!(content, include_str!("../examples/greek.pdf"));
}
