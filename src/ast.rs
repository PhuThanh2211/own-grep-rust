#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(char),
    Any,                                       // .
    Digit,                                     // \d
    Word,                                      // \w
    Group { chars: Vec<char>, negated: bool }, // [abc], [^abc]
}

impl Token {
    pub fn matches(&self, c: char) -> bool {
        match self {
            Token::Literal(l) => *l == c,
            Token::Any => c != '\n',
            Token::Digit => c.is_ascii_digit(),
            Token::Word => c.is_ascii_alphanumeric() || c == '_',
            Token::Group { chars, negated } => chars.contains(&c) != *negated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quant {
    pub min: usize,
    pub max: Option<usize>
}

impl Quant {
    pub const ONE: Quant = Quant { min: 1, max: Some(1) };
    pub const ONE_OR_MORE: Quant = Quant { min: 1, max: None };
    pub const ZERO_OR_ONE: Quant = Quant { min: 0, max: Some(1) };
}

#[derive(Debug, PartialEq)]
pub struct Piece {
    pub token: Token,
    pub quant: Quant
}
