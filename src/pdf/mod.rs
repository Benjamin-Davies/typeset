use std::{
    collections::BTreeMap,
    fmt::{self, Write},
};

use crate::font::Font;

use self::page::{PAGE_HEIGHT, PAGE_WIDTH};

pub mod page;

const HEADER: &str = "%PDF-1.7\n";

pub struct PDFBuilder {
    content: String,
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

    fn start_object(&mut self) -> Result<Ref, fmt::Error> {
        let ref_ = self.preallocate_object();
        self.start_object_with_ref(ref_)?;
        Ok(ref_)
    }

    fn start_object_with_ref(&mut self, ref_: Ref) -> Result<(), fmt::Error> {
        let Ref(id, generation) = ref_;

        let offset = self.content.len() as u32;
        self.xref[id as usize] = XRefEntry::InUse { offset, generation };

        writeln!(self.content, "{id} {generation} obj")?;

        Ok(())
    }

    fn end_object(&mut self) -> Result<(), fmt::Error> {
        writeln!(self.content, "endobj")
    }

    fn stream_object(&mut self, content: &str) -> Result<Ref, fmt::Error> {
        let ref_ = self.start_object()?;
        writeln!(self.content, "<< /Length {} >>", content.len())?;
        writeln!(self.content, "stream")?;
        self.content.push_str(content);
        write!(self.content, "\nendstream\n")?;
        self.end_object()?;
        Ok(ref_)
    }

    fn font(&mut self, font: &Font) -> Result<Ref, fmt::Error> {
        let len = font.data.len();
        let encoded = ascii85_encode(font.data);
        let encoded_len = encoded.len();

        let font_file2 = self.start_object()?;
        writeln!(
            self.content,
            "<< /Length {encoded_len} /Length1 {len} /Filter /ASCII85Decode >>",
        )?;
        writeln!(self.content, "stream")?;
        self.content.push_str(&encoded);
        write!(self.content, "\nendstream\n")?;
        self.end_object()?;

        let font_descriptor = self.start_object()?;
        write!(
            self.content,
            "<< /Type /FontDescriptor /FontName /{ps_name} /Flags 6 ",
            ps_name = font.ps_name,
        )?;
        let bbox = font.face.global_bounding_box();
        write!(
            self.content,
            "/FontBBox [{x1} {y1} {x2} {y2}] /ItalicAngle {angle} ",
            x1 = font.to_milli_em(bbox.x_min),
            y1 = font.to_milli_em(bbox.y_min),
            x2 = font.to_milli_em(bbox.x_max),
            y2 = font.to_milli_em(bbox.y_max),
            angle = font.face.italic_angle().unwrap_or(0.0),
        )?;
        write!(
            self.content,
            "/Ascent {ascent} /Descent {descent} ",
            ascent = font.to_milli_em(font.face.ascender()),
            descent = font.to_milli_em(font.face.descender()),
        )?;
        write!(
            self.content,
            "/Leading {leading} /CapHeight {cap_height} /StemV {stem_v} /FontFile2 {font_file2} >>",
            cap_height = font.to_milli_em(font.face.ascender()),
            leading = font
                .to_milli_em(font.face.line_gap() + font.face.ascender() - font.face.descender()),
            stem_v = 100,
            font_file2 = font_file2,
        )?;
        self.end_object()?;

        let widths_ref = self.start_object()?;
        write!(self.content, "[ ")?;
        for width in &font.widths {
            write!(self.content, "{width} ")?;
        }
        write!(self.content, "]")?;
        self.end_object()?;

        let font_ref = self.start_object()?;
        write!(
            self.content,
            "<< /Type /Font /Subtype /TrueType /FirstChar {first_char} /LastChar {last_char} ",
            first_char = *font.char_range.start() as u32,
            last_char = *font.char_range.end() as u32,
        )?;
        write!(
            self.content,
            "/Widths {widths_ref} /FontDescriptor {font_descriptor} >>",
        )?;
        self.end_object()?;

        Ok(font_ref)
    }

    pub fn page(&mut self, content: &str) -> Result<(), fmt::Error> {
        let contents = self.stream_object(content)?;

        let page = self.start_object()?;
        write!(
            self.content,
            "<< /Type /Page /Parent {pages} /Contents {contents} >>",
            pages = self.pages_ref,
        )?;
        self.end_object()?;

        self.page_refs.push(page);

        Ok(())
    }

    pub fn catalog(&mut self, fonts: &BTreeMap<&str, &Font>) -> Result<(), fmt::Error> {
        let font_refs = fonts
            .iter()
            .map(|(ps_name, font)| {
                let ref_ = self.font(font)?;
                Ok((ps_name, ref_))
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.start_object_with_ref(self.pages_ref)?;
        write!(self.content, "<< /Type /Pages /Kids [ ")?;
        for page_ref in &self.page_refs {
            write!(self.content, "{page_ref} ")?;
        }
        write!(
            self.content,
            "] /Count {page_count} ",
            page_count = self.page_refs.len(),
        )?;
        write!(self.content, "/Resources << /Font <<",)?;
        for (ps_name, font_ref) in font_refs {
            write!(self.content, "/{ps_name} {font_ref} ")?;
        }
        write!(
            self.content,
            ">> >> /MediaBox [ 0 0 {PAGE_WIDTH} {PAGE_HEIGHT} ] >>",
        )?;
        self.end_object()?;

        let catalog = self.start_object()?;
        write!(
            self.content,
            "<< /Type /Catalog /Pages {pages} >>",
            pages = self.pages_ref,
        )?;
        self.end_object()?;

        self.root = catalog;

        Ok(())
    }

    pub fn build(self) -> Result<String, fmt::Error> {
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
        writeln!(content, "xref")?;
        writeln!(content, "0 {xref_size}")?;
        for entry in xref {
            let (n, g, c) = match entry {
                XRefEntry::Free {
                    next_free,
                    generation,
                } => (next_free, generation, 'f'),
                XRefEntry::InUse { offset, generation } => (offset, generation, 'n'),
            };
            write!(content, "{n:010} {g:05} {c}\r\n")?;
        }

        writeln!(content, "trailer")?;
        writeln!(content, "<< /Size {xref_size} /Root {root} >>")?;

        writeln!(content, "startxref")?;
        writeln!(content, "{start_xref}")?;
        writeln!(content, "%%EOF")?;

        Ok(content)
    }
}

fn ascii85_encode(data: &[u8]) -> String {
    let encoded = ascii85::encode(data);
    let encoded = &encoded[2..];

    // Break into 78-character lines
    const MAX_LEN: usize = 78;
    let mut rest = encoded;
    let mut s = String::new();
    while rest.len() > MAX_LEN {
        let prefix;
        (prefix, rest) = rest.split_at(MAX_LEN);
        s.push_str(prefix);
        s.push('\n');
    }
    s.push_str(rest);

    s
}
