use std::{fmt, io::Write};

use crate::font::Font;

use self::page::{PAGE_HEIGHT, PAGE_WIDTH};

pub mod page;

const HEADER: &[u8] = b"%PDF-1.7\n";

pub struct PDFBuilder {
    content: Vec<u8>,
    xref: Vec<XRefEntry>,
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

impl PDFBuilder {
    pub fn new() -> Self {
        Self {
            content: HEADER.to_owned(),
            xref: vec![XRefEntry::Free {
                // Will be filled in when XREF table is generated
                next_free: 0,
                generation: u16::MAX,
            }],
            root: Ref(0, 0),
        }
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

        write!(self.content, "{id} {generation} obj\n").unwrap();
    }

    fn end_object(&mut self) {
        write!(self.content, "endobj\n").unwrap();
    }

    fn stream_object(&mut self, content: &[u8]) -> Ref {
        let ref_ = self.start_object();
        write!(self.content, "<< /Length {} >>\n", content.len()).unwrap();
        write!(self.content, "stream\n").unwrap();
        self.content.extend_from_slice(content);
        write!(self.content, "\nendstream\n").unwrap();
        self.end_object();
        ref_
    }

    fn font(&mut self, font: &Font) -> Ref {
        let len = font.data.len();
        let font_file2 = self.start_object();
        write!(self.content, "<< /Length {len} /Length1 {len} >>\n",).unwrap();
        write!(self.content, "stream\n").unwrap();
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
            "/FontBBox [ {x1} {y1} {x2} {y2} ] /ItalicAngle {angle} /Ascent {ascent} /Descent {descent} ",
            x1 = bbox.x_min,
            y1 = bbox.y_min,
            x2 = bbox.x_max,
            y2 = bbox.y_max,
            angle = font.face.italic_angle().unwrap_or(0.0),
            ascent = font.face.ascender(),
            descent = font.face.descender(),
        )
        .unwrap();
        write!(
            self.content,
            "/CapHeight {cap_height} /StemV {stem_v} /FontFile2 {font_file2} >>",
            cap_height = font.face.height(),
            stem_v = 100,
            font_file2 = font_file2,
        )
        .unwrap();
        self.end_object();

        let font = self.start_object();
        write!(
            self.content,
            "<< /Type /Font /Subtype /TrueType /FontDescriptor {font_descriptor} >>"
        )
        .unwrap();
        self.end_object();

        font
    }

    pub fn single_page(mut self, content: &[u8]) -> Self {
        let contents = self.stream_object(content);

        let pages = self.preallocate_object();
        let page = self.start_object();
        write!(
            self.content,
            "<< /Type /Page /Parent {pages} /Contents {contents} >>"
        )
        .unwrap();
        self.end_object();

        let font = Font::default();
        let font_ref = self.font(&font);

        self.start_object_with_ref(pages);
        write!(self.content, "<< /Type /Pages /Kids [ {page} ] /Count 1 ").unwrap();
        write!(
            self.content,
            "/Resources << /Font << /{ps_name} {font_ref} >> >> /MediaBox [ 0 0 {PAGE_WIDTH} {PAGE_HEIGHT} ] >>",
            ps_name = font.ps_name,
        ).unwrap();
        self.end_object();

        let catalog = self.start_object();
        write!(self.content, "<< /Type /Catalog /Pages {pages} >>").unwrap();
        self.end_object();

        self.root = catalog;

        self
    }

    pub fn build(self) -> Vec<u8> {
        let Self {
            mut content,
            mut xref,
            root,
        } = self;

        let xref_size = xref.len() as u32;
        xref[0] = XRefEntry::Free {
            next_free: xref_size,
            generation: u16::MAX,
        };

        let start_xref = content.len();
        write!(content, "xref\n").unwrap();
        write!(content, "0 {xref_size}\n").unwrap();
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

        write!(content, "trailer\n").unwrap();
        write!(content, "<< /Size {xref_size} /Root {root} >>\n").unwrap();

        write!(content, "startxref\n").unwrap();
        write!(content, "{start_xref}\n").unwrap();
        write!(content, "%%EOF\n").unwrap();

        content
    }
}
