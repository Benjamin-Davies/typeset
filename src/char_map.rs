use crate::document::{Block, Document, Inline};

/// Defines a mapping from document-specific character numbers to Unicode code points.
#[derive(Debug)]
pub struct CharMap {
    pub mappings: Vec<char>,
}

impl CharMap {
    pub fn from_document(document: &Document) -> Self {
        let mut char_map = Self {
            mappings: vec!['\0'],
        };
        char_map.extend(&document.blocks);
        char_map
    }

    pub fn insert(&mut self, c: char) {
        if !self.mappings.contains(&c) {
            // This is a temporary restriction to avoid having to deal with
            // multi-byte characters.
            // TODO: Remove this restriction
            assert!(self.mappings.len() <= u8::MAX as usize, "CharMap is full");
            self.mappings.push(c);
        }
    }

    pub fn get(&self, c: char) -> Option<u8> {
        self.mappings.iter().position(|&x| x == c).map(|x| x as u8)
    }
}

impl<'a> Extend<&'a Block<'a>> for CharMap {
    fn extend<T: IntoIterator<Item = &'a Block<'a>>>(&mut self, iter: T) {
        for block in iter {
            match block {
                Block::Text(text_block) => {
                    self.extend(&text_block.inlines);
                }
            }
        }
    }
}

impl<'a> Extend<&'a Inline<'a>> for CharMap {
    fn extend<T: IntoIterator<Item = &'a Inline<'a>>>(&mut self, iter: T) {
        for inline in iter {
            self.extend(inline.text.chars());
        }
    }
}

impl Extend<char> for CharMap {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for c in iter {
            self.insert(c);
        }
    }
}
