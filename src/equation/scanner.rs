use std::fmt;

#[derive(Debug)]
pub struct LatexScanner<'a> {
    source: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Keyword(&'a str),
    Char(char),
}

impl<'a> LatexScanner<'a> {
    pub fn new(source: &'a str) -> Self {
        LatexScanner { source }
    }
}

impl<'a> Iterator for LatexScanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        loop {
            let next = self.source.chars().next()?;
            if next.is_whitespace() {
                self.source = &self.source[next.len_utf8()..];
            } else {
                break;
            }
        }

        let next = self.source.chars().next()?;
        if next == '\\' {
            self.source = &self.source[next.len_utf8()..];
            let len = self
                .source
                .find(|c: char| !c.is_alphabetic())
                .unwrap_or(self.source.len());
            let keyword = &self.source[..len];
            self.source = &self.source[len..];
            Some(Token::Keyword(keyword))
        } else {
            self.source = &self.source[next.len_utf8()..];
            Some(Token::Char(next))
        }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "\\{keyword}"),
            Token::Char(c) => write!(f, "{c}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let scanner = LatexScanner::new(r"\alpha + \beta");
        let tokens: Vec<_> = scanner.collect();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword("alpha"),
                Token::Char('+'),
                Token::Keyword("beta")
            ]
        );
    }
}
