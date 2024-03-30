use std::collections::BTreeMap;

use glam::Vec2;

use crate::font::Font;

pub struct Document<'a> {
    pub blocks: Vec<Block<'a>>,
    pub fonts: BTreeMap<&'a str, &'a Font<'a>>,
    pub page_size: Vec2,
    pub margin: f32,
}

pub enum Block<'a> {
    Text(TextBlock<'a>),
}

pub struct TextBlock<'a> {
    pub inlines: Vec<Inline<'a>>,
    pub align: TextAlign,
}

pub struct Inline<'a> {
    pub text: &'a str,
    pub style: Style<'a>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Style<'a> {
    pub font: &'a str,
    pub font_size: f32,
}

pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}
