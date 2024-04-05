use std::iter::Peekable;

use crate::equation::Delimiter;

use super::{
    scanner::{LatexScanner, Token},
    Fragment, Sequence,
};

pub fn parse_latex(source: &str) -> Sequence {
    let mut scanner = LatexScanner::new(source).peekable();

    let sequence = parse_sequence(&mut scanner);
    assert_eq!(scanner.next(), None, "Unexpected token");

    sequence
}

fn parse_sequence(scanner: &mut Peekable<LatexScanner>) -> Sequence {
    let mut fragments = Vec::new();

    while let Some(fragment) = parse_fragment(scanner) {
        fragments.push(fragment);
    }

    Sequence { fragments }
}

fn parse_fragment(scanner: &mut Peekable<LatexScanner>) -> Option<Fragment> {
    match *scanner.peek()? {
        Token::Keyword(keyword) => match keyword {
            "left" => {
                scanner.next();
                let delimiter = match scanner.next() {
                    Some(Token::Char('(')) => Delimiter::Paren,
                    Some(Token::Char('[')) => Delimiter::Bracket,
                    Some(Token::Char('{')) => Delimiter::Brace,
                    _ => panic!("Unknown delimiter"),
                };
                let right_delimiter = match delimiter {
                    Delimiter::Paren => ')',
                    Delimiter::Bracket => ']',
                    Delimiter::Brace => '}',
                };

                let sequence = parse_sequence(scanner);

                assert_eq!(scanner.next(), Some(Token::Keyword("right")));
                assert_eq!(scanner.next(), Some(Token::Char(right_delimiter)));

                Some(Fragment::Delimited(Box::new((delimiter, sequence))))
            }
            "right" => None,
            "frac" => {
                scanner.next();
                let numerator = parse_group(scanner);
                let denominator = parse_group(scanner);
                Some(Fragment::Fraction(Box::new((numerator, denominator))))
            }
            _ => {
                if let Some(c) = parse_keyword_symbol(keyword) {
                    scanner.next();
                    Some(Fragment::Char(c))
                } else {
                    panic!("Unknown keyword: \\{keyword}");
                }
            }
        },
        Token::Char(c) => match c {
            '{' => {
                let sequence = parse_group(scanner);
                Some(Fragment::Group(Box::new(sequence)))
            }
            '}' => None,
            '^' => {
                scanner.next();
                let fragment = parse_fragment(scanner)?;
                Some(Fragment::Superscript(Box::new(fragment)))
            }
            '_' => {
                scanner.next();
                let fragment = parse_fragment(scanner)?;
                Some(Fragment::Subscript(Box::new(fragment)))
            }
            _ => {
                scanner.next();
                Some(Fragment::Char(c))
            }
        },
    }
}

fn parse_group(scanner: &mut Peekable<LatexScanner>) -> Sequence {
    assert_eq!(scanner.next(), Some(Token::Char('{')));
    let sequence = parse_sequence(scanner);
    assert_eq!(scanner.next(), Some(Token::Char('}')));

    sequence
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e_mc2() {
        let sequence = parse_latex(r"E = mc^2");
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
            parse_latex(r"\alpha_\beta(\gamma, \theta) = \frac{\beta}{\gamma} \cdot e^{i\theta}");
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
        );
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
