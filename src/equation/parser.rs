use std::iter::Peekable;

use thiserror::Error;

use crate::equation::Delimiter;

use super::{
    scanner::{LatexScanner, Token},
    Fragment, Sequence,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("expected end of file")]
    ExpectedEof,
    #[error("expected fragment")]
    ExpectedFragment,
    #[error("expected token: '{0}'")]
    ExpectedToken(String),
    #[error("invalid delimiter: '{0}'")]
    InvalidDelimiter(String),
    #[error("unknown keyword: '\\{0}'")]
    UnknownKeyword(String),
}

pub fn parse_latex(source: &str) -> Result<Sequence, Error> {
    let mut scanner = LatexScanner::new(source).peekable();

    let sequence = parse_sequence(&mut scanner)?;
    if scanner.peek().is_some() {
        return Err(Error::ExpectedEof);
    }

    Ok(sequence)
}

fn parse_sequence(scanner: &mut Peekable<LatexScanner>) -> Result<Sequence, Error> {
    let mut fragments = Vec::new();

    while let Some(fragment) = parse_fragment(scanner).map_or_else(
        |err| match err {
            Error::ExpectedFragment => Ok(None),
            _ => Err(err),
        },
        |x| Ok(Some(x)),
    )? {
        fragments.push(fragment);
    }

    Ok(Sequence { fragments })
}

fn parse_fragment(scanner: &mut Peekable<LatexScanner>) -> Result<Fragment, Error> {
    match *scanner.peek().ok_or(Error::ExpectedFragment)? {
        Token::Keyword(keyword) => match keyword {
            "left" => {
                scanner.next();
                let delimiter = match scanner.next().ok_or(Error::UnexpectedEof)? {
                    Token::Char('(') => Delimiter::Paren,
                    Token::Char('[') => Delimiter::Bracket,
                    Token::Char('{') => Delimiter::Brace,
                    token => return Err(Error::InvalidDelimiter(token.to_string())),
                };
                let right_delimiter = match delimiter {
                    Delimiter::Paren => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                };

                let sequence = parse_sequence(scanner)?;

                expect_token(scanner, Token::Keyword("right"))?;
                expect_token(scanner, Token::Char(right_delimiter))?;

                Ok(Fragment::Delimited(Box::new((delimiter, sequence))))
            }
            "right" => Err(Error::ExpectedFragment),
            "frac" => {
                scanner.next();
                let numerator = parse_group(scanner)?;
                let denominator = parse_group(scanner)?;
                Ok(Fragment::Fraction(Box::new((numerator, denominator))))
            }
            _ => {
                if let Some(c) = parse_keyword_symbol(keyword) {
                    scanner.next();
                    Ok(Fragment::Char(c))
                } else {
                    Err(Error::UnknownKeyword(keyword.to_owned()))
                }
            }
        },
        Token::Char(c) => match c {
            '{' => {
                let sequence = parse_group(scanner)?;
                Ok(Fragment::Group(Box::new(sequence)))
            }
            '}' => Err(Error::ExpectedFragment),
            '^' => {
                scanner.next();
                let fragment = parse_fragment(scanner)?;
                Ok(Fragment::Superscript(Box::new(fragment)))
            }
            '_' => {
                scanner.next();
                let fragment = parse_fragment(scanner)?;
                Ok(Fragment::Subscript(Box::new(fragment)))
            }
            _ => {
                scanner.next();
                Ok(Fragment::Char(c))
            }
        },
    }
}

fn parse_group(scanner: &mut Peekable<LatexScanner>) -> Result<Sequence, Error> {
    expect_token(scanner, Token::Char('{'))?;
    let sequence = parse_sequence(scanner)?;
    expect_token(scanner, Token::Char('}'))?;

    Ok(sequence)
}

fn parse_keyword_symbol(keyword: &str) -> Option<char> {
    match keyword {
        "alpha" => Some('α'),
        "beta" => Some('β'),
        "gamma" => Some('γ'),
        "delta" => Some('δ'),
        "epsilon" => Some('ε'),
        "zeta" => Some('ζ'),
        "eta" => Some('η'),
        "theta" => Some('θ'),
        "iota" => Some('ι'),
        "kappa" => Some('κ'),
        "lambda" => Some('λ'),
        "mu" => Some('μ'),
        "nu" => Some('ν'),
        "xi" => Some('ξ'),
        "omicron" => Some('ο'),
        "pi" => Some('π'),
        "rho" => Some('ρ'),
        "sigma" => Some('σ'),
        "tau" => Some('τ'),
        "upsilon" => Some('υ'),
        "phi" => Some('φ'),
        "chi" => Some('χ'),
        "psi" => Some('ψ'),
        "omega" => Some('ω'),
        "Alpha" => Some('Α'),
        "Beta" => Some('Β'),
        "Gamma" => Some('Γ'),
        "Delta" => Some('Δ'),
        "Epsilon" => Some('Ε'),
        "Zeta" => Some('Ζ'),
        "Eta" => Some('Η'),
        "Theta" => Some('Θ'),
        "Iota" => Some('Ι'),
        "Kappa" => Some('Κ'),
        "Lambda" => Some('Λ'),
        "Mu" => Some('Μ'),
        "Nu" => Some('Ν'),
        "Xi" => Some('Ξ'),
        "Omicron" => Some('Ο'),
        "Pi" => Some('Π'),
        "Rho" => Some('Ρ'),
        "Sigma" => Some('Σ'),
        "Tau" => Some('Τ'),
        "Upsilon" => Some('Υ'),
        "Phi" => Some('Φ'),
        "Chi" => Some('Χ'),
        "Psi" => Some('Ψ'),
        "Omega" => Some('Ω'),

        "cdot" => Some('⋅'),
        "times" => Some('×'),
        "div" => Some('÷'),
        "pm" => Some('±'),
        "mp" => Some('∓'),
        "leq" => Some('≤'),
        "geq" => Some('≥'),
        "neq" => Some('≠'),
        "approx" => Some('≈'),
        "equiv" => Some('≡'),
        "forall" => Some('∀'),
        "exists" => Some('∃'),
        "in" => Some('∈'),
        "notin" => Some('∉'),
        "subset" => Some('⊂'),
        "supset" => Some('⊃'),
        "subseteq" => Some('⊆'),
        "supseteq" => Some('⊇'),
        "emptyset" => Some('∅'),
        "nabla" => Some('∇'),
        "partial" => Some('∂'),
        "infty" => Some('∞'),
        "aleph" => Some('ℵ'),

        "{" => Some('{'),
        "}" => Some('}'),
        "^" => Some('^'),
        "_" => Some('_'),

        _ => None,
    }
}

pub fn expect_token(scanner: &mut Peekable<LatexScanner>, expected: Token) -> Result<(), Error> {
    match scanner.next() {
        Some(token) if token == expected => Ok(()),
        Some(token) => Err(Error::ExpectedToken(token.to_string())),
        None => Err(Error::UnexpectedEof),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e_mc2() {
        let sequence = parse_latex(r"E = mc^2").unwrap();
        assert_eq!(
            sequence,
            Sequence {
                fragments: vec![
                    Fragment::Char('E'),
                    Fragment::Char('='),
                    Fragment::Char('m'),
                    Fragment::Char('c'),
                    Fragment::Superscript(Box::new(Fragment::Char('2'))),
                ],
            },
        );
    }

    #[test]
    fn test_greek_frac() {
        let sequence =
            parse_latex(r"\alpha_\beta(\gamma, \theta) = \frac{\beta}{\gamma} \cdot e^{i\theta}")
                .unwrap();
        assert_eq!(
            sequence,
            Sequence {
                fragments: vec![
                    Fragment::Char('α'),
                    Fragment::Subscript(Box::new(Fragment::Char('β'))),
                    Fragment::Char('('),
                    Fragment::Char('γ'),
                    Fragment::Char(','),
                    Fragment::Char('θ'),
                    Fragment::Char(')'),
                    Fragment::Char('='),
                    Fragment::Fraction(Box::new((
                        Sequence {
                            fragments: vec![Fragment::Char('β')],
                        },
                        Sequence {
                            fragments: vec![Fragment::Char('γ')],
                        },
                    ))),
                    Fragment::Char('⋅'),
                    Fragment::Char('e'),
                    Fragment::Superscript(Box::new(Fragment::Group(Box::new(Sequence {
                        fragments: vec![Fragment::Char('i'), Fragment::Char('θ')],
                    })))),
                ],
            },
        );
    }

    #[test]
    fn test_big_delim() {
        let sequence = parse_latex(
            r"\alpha_\beta(\gamma, \theta) = \left[\frac{\beta}{\gamma}\right] \cdot \left(e^{i\theta}\right)",
        ).unwrap();
        assert_eq!(
            sequence,
            Sequence {
                fragments: vec![
                    Fragment::Char('α'),
                    Fragment::Subscript(Box::new(Fragment::Char('β'))),
                    Fragment::Char('('),
                    Fragment::Char('γ'),
                    Fragment::Char(','),
                    Fragment::Char('θ'),
                    Fragment::Char(')'),
                    Fragment::Char('='),
                    Fragment::Delimited(Box::new((
                        Delimiter::Bracket,
                        Sequence {
                            fragments: vec![Fragment::Fraction(Box::new((
                                Sequence {
                                    fragments: vec![Fragment::Char('β')],
                                },
                                Sequence {
                                    fragments: vec![Fragment::Char('γ')],
                                },
                            )))],
                        }
                    ))),
                    Fragment::Char('⋅'),
                    Fragment::Delimited(Box::new((
                        Delimiter::Paren,
                        Sequence {
                            fragments: vec![
                                Fragment::Char('e'),
                                Fragment::Superscript(Box::new(Fragment::Group(Box::new(
                                    Sequence {
                                        fragments: vec![Fragment::Char('i'), Fragment::Char('θ')],
                                    },
                                )))),
                            ],
                        },
                    ))),
                ],
            },
        );
    }
}
