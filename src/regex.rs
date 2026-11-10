use crate::ast::Piece;
use crate::parser::Parser;

pub struct Regex {
    pub(crate) pieces: Vec<Piece>,
    pub(crate) start_anchored: bool,
    pub(crate) end_anchored: bool,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Regex, String> {
        Parser::new(pattern).parse()
    }
    pub fn is_match(&self, input: &str) -> bool {
        let input: Vec<char> = input.chars().collect();
        if self.start_anchored {
            self.match_here(&input, &self.pieces)
        } else {
            (0..=input.len()).any(|start| self.match_here(&input[start..], &self.pieces))
        }
    }
    pub fn match_here(&self, input: &[char], pieces: &[Piece]) -> bool {
        let Some((piece, rest)) = pieces.split_first() else {
            return !self.end_anchored || input.is_empty();
        };

        // greedy: take as many matching chars as allowed
        let limit = piece.quant.max.unwrap_or(usize::MAX);
        let run = input
            .iter()
            .take(limit)
            .take_while(|&&c| piece.token.matches(c))
            .count();

        if run < piece.quant.min {
            return false;
        }
        // back off one char at a time until the rest matches
        (piece.quant.min..=run)
            .rev()
            .any(|n| self.match_here(&input[n..], rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn m(input: &str, pat: &str) -> bool {
        Regex::new(pat).unwrap().is_match(input)
    }

    #[test]
    fn digit_class() {
        assert!(m("foo101", r"\d"));
        assert!(!m("Art", r"\d"));
    }

    #[test]
    fn word_class() {
        assert!(m("alpha_num3ric", r"\w"));
        assert!(m("_", r"\w"));
        assert!(!m("$!?", r"\w"));
    }

    #[test]
    fn positive_group() {
        assert!(m("apple", "[abc]"));
        assert!(m("cab", "[abc]"));
        assert!(!m("dog", "[abc]"));
        assert!(m("a1b2c3", "[123]"));
    }

    #[test]
    fn negative_group() {
        assert!(m("cat", "[^abc]"));
        assert!(!m("cab", "[^abc]"));
        assert!(m("apple", "[^abc]"));
    }

    #[test]
    fn sequences() {
        assert!(m("1 apple", r"\d apple"));
        assert!(!m("1 orange", r"\d apple"));
        assert!(m("I got 100 apples from the store", r"\d\d\d apples"));
        assert!(!m("I got 1 apple from the store", r"\d\d\d apples"));
        assert!(m("4 cats", r"\d \w\w\ws"));
        assert!(!m("1 dog", r"\d \w\w\ws"));
    }

    #[test]
    fn start_anchor() {
        assert!(m("log", "^log"));
        assert!(m("logs", "^log"));
        assert!(!m("slog", "^log"));
        assert!(m("123abc", r"^\d\d\d"));
        assert!(m("cat", "[^abc]"));
    }

    #[test]
    fn end_anchor() {
        assert!(m("dog", "dog$"));
        assert!(m("hotdog", "dog$"));
        assert!(!m("dogs", "dog$"));
        assert!(m("abc123", r"\d\d\d$"));
        assert!(!m("abc123@", r"\w\w\w$"));
        assert!(m("abc123cde", r"\w\w\w$"));
        assert!(m("dog", "^dog$"));
        assert!(!m("hotdog", "^dog$"));
    }

    #[test]
    fn one_or_more() {
        assert!(m("apple", "a+"));
        assert!(m("SaaS", "a+"));
        assert!(!m("dog", "a+"));
        assert!(m("cats", "ca+ts"));
        assert!(m("caats", "ca+ts"));
        assert!(!m("cts", "ca+ts"));
        assert!(m("123", r"\d+"));
        assert!(m("cats", r"\w+s"));     // needs back off
        assert!(m("caaats", "^ca+ts$"));
    }

    #[test]
    fn escaping_and_errors() {
        assert!(m("cost $5", r"\$5"));
        assert!(m("a$", r"a\$$"));
        assert!(Regex::new("[abc").is_err());
        assert!(Regex::new(r"abc\").is_err());
    }
}