# Typeset

A small experiment in positioning text on a page. So far I've implemented line wrapping, justification and font mixing. Eventually I might add support for formulas in addition to prose.

Running `cargo test` downloads Noto Serif and generates some test PDFs in the `output` folder.

I've written my own code for generating PDFs and subsetting the TrueType fonts, but have not had the chance to properly debug it on macOS. The subsetted font displays correctly in PDF.js (most browsers) and Gnome Evince, but not in macOS Preview. The last version that worked with Preview is in the [macos branch](https://github.com/Benjamin-Davies/typeset/tree/macos).

## TODO

- [x] X-positions: Given a string of glyphs, I want to generate a list of X-positions for when the characters are printed in a line.
- [x] PDF generation: Given a string of glyphs and their positions, I want to hand-generate a minimal PDF with those glyphs displayed and that font embedded. The PDF should be A4 with 72pt margins.
- [x] Soft wrap: Given a string of glyphs, I want to insert soft line breaks between words so that the text is left aligned.
- [x] Paragraphs: Given a list of paragraphs of glyphs, I want to draw them as blocks with some padding between them.
- [x] Justified text: Given a list of paragraphs of glyphs, I want the text within each paragraph to be justified.
- [x] Inline formatting: Given a list of blocks, I want to be able to assign a different font-size, weight or emphasis to any sequence of glyphs and have their base-lines line up.
- [x] Page breaks: Given a document AST, when I render it and it needs more than one page it should break the content into pages.
- [x] Write unit tests and refactor
- [x] Unicode support
- [x] Only include the glyphs that are actually used in the PDF
- [x] Formula parsing: Given a LaTeX fomula, I want to parse an AST.
- [ ] Formula drawing: Given a LaTeX formula AST, I want to draw it as a block in combination with text blocks.
