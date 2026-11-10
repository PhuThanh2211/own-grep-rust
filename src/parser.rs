use crate::ast::{Piece, Quant, Token};
use crate::regex::Regex;

pub struct Parser {
    chars: Vec<char>,
    pos: usize
}

impl Parser {
    pub fn new(pattern: &str) -> Self {
        Parser { chars: pattern.chars().collect(), pos: 0 }
    }

    pub fn parse(mut self) -> Result<Regex, String> {
        let start_anchored = self.eat('^');
        let mut end_anchored = false;
        let mut pieces = Vec::new();

        while let Some(c) = self.next() {
            if c == '$' && self.peek().is_none() {
                end_anchored = true;
                break;
            }
            let token = self.parse_token(c)?;
            let quant = self.parse_quant();
            pieces.push(Piece { token, quant });
        }

        Ok(Regex { pieces, start_anchored, end_anchored })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_token(&mut self, c: char) -> Result<Token, String> {
        Ok(match c {
            '\\' => match self.next().ok_or("trailing backslash")? {
                'd' => Token::Digit,
                'w' => Token::Word,
                other => Token::Literal(other), // \\ \[ \$ \+ ...
            },
            '[' => self.parse_group()?,
            c => Token::Literal(c),
        })
    }

    fn parse_group(&mut self) -> Result<Token, String> {
        let negated = self.eat('^');
        let mut chars = Vec::new();
        loop {
            match self.next() {
                Some(']') => break,
                Some(c) => chars.push(c),
                None => return Err("unclosed [".into()),
            }
        }
        Ok(Token::Group { chars, negated })
    }

    fn parse_quant(&mut self) -> Quant {
        if self.eat('+') {
            Quant::ONE_OR_MORE
        } else {
            Quant::ONE
        }
    }
}