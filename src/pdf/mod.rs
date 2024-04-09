use std::{collections::BTreeMap, fmt, io::Write};

use crate::font::Font;

use self::page::{PAGE_HEIGHT, PAGE_WIDTH};

pub mod page;

const HEADER: &[u8] = b"%PDF-1.7\n";

pub struct PDFBuilder {
    content: Vec<u8>,
    xref: Vec<XRefEntry>,
    pages_ref: Ref,
    page_refs: Vec<Ref>,
    root: Ref,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Ref(u32, u16);

#[derive(Debug)]
enum XRefEntry {
    Free { next_free: u32, generation: u16 },
    InUse { offset: u32, generation: u16 },
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} R", self.0, self.1)
    }
}

impl Default for PDFBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PDFBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            content: HEADER.to_owned(),
            xref: vec![XRefEntry::Free {
                // Will be filled in when XREF table is generated
                next_free: 0,
                generation: u16::MAX,
            }],
            pages_ref: Ref::default(),
            page_refs: Vec::new(),
            root: Ref::default(),
        };
        builder.pages_ref = builder.preallocate_object();
        builder
    }

    fn preallocate_object(&mut self) -> Ref {
        let id = self.xref.len() as u32;
        self.xref.push(XRefEntry::Free {
            next_free: 0,
            generation: u16::MAX,
        });
        Ref(id, 0)
    }

    fn start_object(&mut self) -> Ref {
        let ref_ = self.preallocate_object();
        self.start_object_with_ref(ref_);
        ref_
    }

    fn start_object_with_ref(&mut self, ref_: Ref) {
        let Ref(id, generation) = ref_;

        let offset = self.content.len() as u32;
        self.xref[id as usize] = XRefEntry::InUse { offset, generation };

        writeln!(self.content, "{id} {generation} obj").unwrap();
    }

    fn end_object(&mut self) {
        writeln!(self.content, "endobj").unwrap();
    }

    fn stream_object(&mut self, content: &[u8]) -> Ref {
        let ref_ = self.start_object();
        writeln!(self.content, "<< /Length {} >>", content.len()).unwrap();
        writeln!(self.content, "stream").unwrap();
        self.content.extend_from_slice(content);
        write!(self.content, "\nendstream\n").unwrap();
        self.end_object();
        ref_
    }

    fn font(&mut self, font: &Font) -> Ref {
        let len = font.data.len();
        let font_file2 = self.start_object();
        writeln!(self.content, "<< /Length {len} /Length1 {len} >>",).unwrap();
        writeln!(self.content, "stream").unwrap();
        self.content.extend_from_slice(font.data);
        write!(self.content, "\nendstream\n").unwrap();
        self.end_object();

        let font_descriptor = self.start_object();
        write!(
            self.content,
            "<< /Type /FontDescriptor /FontName /{ps_name} /Flags 6 ",
            ps_name = font.ps_name,
        )
        .unwrap();
        let bbox = font.face.global_bounding_box();
        write!(
            self.content,
            "/FontBBox [{x1} {y1} {x2} {y2}] /ItalicAngle {angle} ",
            x1 = font.to_milli_em(bbox.x_min),
            y1 = font.to_milli_em(bbox.y_min),
            x2 = font.to_milli_em(bbox.x_max),
            y2 = font.to_milli_em(bbox.y_max),
            angle = font.face.italic_angle().unwrap_or(0.0),
        )
        .unwrap();
        write!(
            self.content,
            "/Ascent {ascent} /Descent {descent} ",
            ascent = font.to_milli_em(font.face.ascender()),
            descent = font.to_milli_em(font.face.descender()),
        )
        .unwrap();
        write!(
            self.content,
            "/Leading {leading} /CapHeight {cap_height} /StemV {stem_v} /FontFile2 {font_file2} >>",
            cap_height = font.to_milli_em(font.face.ascender()),
            leading = font
                .to_milli_em(font.face.line_gap() + font.face.ascender() - font.face.descender()),
            stem_v = 100,
            font_file2 = font_file2,
        )
        .unwrap();
        self.end_object();

        let widths_ref = self.start_object();
        write!(self.content, "[ ").unwrap();
        for width in &font.widths {
            write!(self.content, "{width} ").unwrap();
        }
        write!(self.content, "]").unwrap();
        self.end_object();

        let font_ref = self.start_object();
        write!(
            self.content,
            "<< /Type /Font /Subtype /TrueType /FirstChar {first_char} /LastChar {last_char} ",
            first_char = *font.char_range.start() as u32,
            last_char = *font.char_range.end() as u32,
        )
        .unwrap();
        write!(
            self.content,
            "/Widths {widths_ref} /FontDescriptor {font_descriptor} >>",
        )
        .unwrap();
        self.end_object();

        font_ref
    }

    pub fn page(&mut self, content: &[u8]) {
        let contents = self.stream_object(content);

        let page = self.start_object();
        write!(
            self.content,
            "<< /Type /Page /Parent {pages} /Contents {contents} >>",
            pages = self.pages_ref,
        )
        .unwrap();
        self.end_object();

        self.page_refs.push(page);
    }

    pub fn catalog(&mut self, fonts: &BTreeMap<&str, &Font>) {
        let font_refs = fonts
            .iter()
            .map(|(ps_name, font)| {
                let ref_ = self.font(font);
                (ps_name, ref_)
            })
            .collect::<Vec<_>>();

        self.start_object_with_ref(self.pages_ref);
        write!(self.content, "<< /Type /Pages /Kids [ ").unwrap();
        for page_ref in &self.page_refs {
            write!(self.content, "{page_ref} ").unwrap();
        }
        write!(
            self.content,
            "] /Count {page_count} ",
            page_count = self.page_refs.len(),
        )
        .unwrap();
        write!(self.content, "/Resources << /Font <<",).unwrap();
        for (ps_name, font_ref) in font_refs {
            write!(self.content, "/{ps_name} {font_ref} ").unwrap();
        }
        write!(
            self.content,
            ">> >> /MediaBox [ 0 0 {PAGE_WIDTH} {PAGE_HEIGHT} ] >>",
        )
        .unwrap();
        self.end_object();

        let catalog = self.start_object();
        write!(
            self.content,
            "<< /Type /Catalog /Pages {pages} >>",
            pages = self.pages_ref,
        )
        .unwrap();
        self.end_object();

        self.root = catalog;
    }

    pub fn build(self) -> Vec<u8> {
        let Self {
            mut content,
            mut xref,
            root,
            ..
        } = self;

        let xref_size = xref.len() as u32;
        xref[0] = XRefEntry::Free {
            next_free: xref_size,
            generation: u16::MAX,
        };

        let start_xref = content.len();
        writeln!(content, "xref").unwrap();
        writeln!(content, "0 {xref_size}").unwrap();
        for entry in xref {
            let (n, g, c) = match entry {
                XRefEntry::Free {
                    next_free,
                    generation,
                } => (next_free, generation, 'f'),
                XRefEntry::InUse { offset, generation } => (offset, generation, 'n'),
            };
            write!(content, "{n:010} {g:05} {c}\r\n").unwrap();
        }

        writeln!(content, "trailer").unwrap();
        writeln!(content, "<< /Size {xref_size} /Root {root} >>").unwrap();

        writeln!(content, "startxref").unwrap();
        writeln!(content, "{start_xref}").unwrap();
        writeln!(content, "%%EOF").unwrap();

        content
    }
}
