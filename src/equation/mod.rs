pub mod parser;
pub mod scanner;

#[derive(Debug, PartialEq, Eq)]
pub struct Sequence {
    fragments: Vec<Fragment>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Fragment {
    Char(char),
    Group(Box<Sequence>),
    Superscript(Box<Fragment>),
    Subscript(Box<Fragment>),
    Fraction(Box<(Sequence, Sequence)>),
}
