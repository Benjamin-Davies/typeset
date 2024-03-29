# Typeset

This is a simple typesetting program with the aim of generating PDF files with prose and equations.

## TODO

- [x] X-positions: Given a string of glyphs, I want to generate a list of X-positions for when the characters are printed in a line.
- [x] PDF generation: Given a string of glyphs and their positions, I want to hand-generate a minimal PDF with those glyphs displayed and that font embedded. The PDF should be A4 with 72pt margins.
- [x] Soft wrap: Given a string of glyphs, I want to insert soft line breaks between words so that the text is left aligned.
- [x] Paragraphs: Given a list of paragraphs of glyphs, I want to draw them as blocks with some padding between them.
- [x] Justified text: Given a list of paragraphs of glyphs, I want the text within each paragraph to be justified.
- [ ] Inline formatting: Given a list of blocks, I want to be able to assign a different font-size, weight or emphasis to any sequence of glyphs and have their base-lines line up.
- [ ] Formula parsing: Given a LaTeX fomula, I want to parse an AST.
- [ ] Formula drawing: Given a LaTeX formula AST, I want to draw it as a block in combination with text blocks.
