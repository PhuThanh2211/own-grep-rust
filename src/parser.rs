use crate::ast::{Piece, Quant, Token};
use crate::regex::Regex;

pub struct Parser {
    chars: Vec<char>,
    pos: usize,
    end_anchored: bool,
}

impl Parser {
    pub fn new(pattern: &str) -> Self {
        Parser {
            chars: pattern.chars().collect(), pos: 0, end_anchored: false
        }
    }

    pub fn parse(mut self) -> Result<Regex, String> {
        let start_anchored = self.eat('^');
        let mut branches = self.parse_branches()?;

        // parse_branches stops at `)`; at top level that `)` has no matching `(`
        if self.peek().is_some() {
            return Err("unmatched )".into());
        }

        // `cat|dog` without parens: wrap the branches in one Alt piece
        let pieces = if branches.len() == 1 {
            branches.pop().unwrap()
        } else {
            vec![Piece { token: Token::Alt(branches), quant: Quant::ONE }]
        };

        Ok(Regex { pieces, start_anchored, end_anchored: self.end_anchored })
    }

    fn parse_branches(&mut self) -> Result<Vec<Vec<Piece>>, String> {
        let mut branches = vec![self.parse_sequence()?];
        while self.eat('|') {
            branches.push(self.parse_sequence()?);
        }
        Ok(branches)
    }

    fn parse_sequence(&mut self) -> Result<Vec<Piece>, String> {
        let mut pieces = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }

            self.next();
            if c == '$' && self.peek().is_none() {
                self.end_anchored = true;
                break;
            }

            let token = self.parse_token(c)?;
            let quant = self.parse_quant();
            pieces.push(Piece { token, quant });
        }
        Ok(pieces)
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
            '.' => Token::Any,
            '(' => {
              let branches = self.parse_branches()?;
                if !self.eat(')') {
                    return Err("unclosed (".into());
                }
                Token::Alt(branches)
            },
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
        } else if self.eat('?') {
            Quant::ZERO_OR_ONE
        } else {
            Quant::ONE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Piece, Quant, Token};

    fn pieces(p: &str) -> Vec<Piece> {
        Parser::new(p).parse().unwrap().pieces
    }

    #[test]
    fn parses_escapes_and_quantifiers() {
        assert_eq!(
            pieces(r"\d+a"),
            vec![
                Piece { token: Token::Digit, quant: Quant::ONE_OR_MORE },
                Piece { token: Token::Literal('a'), quant: Quant::ONE },
            ]
        );
    }

    #[test]
    fn parses_negated_group() {
        assert_eq!(
            pieces("[^ab]")[0].token,
            Token::Group { chars: vec!['a', 'b'], negated: true }
        );
    }

    #[test]
    fn anchors() {
        let r = Parser::new("^a$").parse().unwrap();
        assert!(r.start_anchored && r.end_anchored);
        let r = Parser::new(r"a\$").parse().unwrap();
        assert!(!r.end_anchored);
    }

    #[test]
    fn dot_wildcard() {
        assert_eq!(pieces(".")[0].token, Token::Any);
        assert_eq!(pieces(r"\.")[0].token, Token::Literal('.'));
    }

    #[test]
    fn errors() {
        assert!(Parser::new("[abc").parse().is_err());
        assert!(Parser::new(r"abc\").parse().is_err());
        assert!(Parser::new("(cat|dog").parse().is_err());
        assert!(Parser::new("cat)").parse().is_err());
    }
}