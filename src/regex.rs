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

    fn check(cases: &[(&str, &str, bool)]) {
        for &(pattern, input, expected) in cases {
            let got = Regex::new(pattern).unwrap().is_match(input);
            assert_eq!(got, expected, "pattern={pattern:?} input={input:?}");
        }
    }

    #[test]
    fn literals_and_classes() {
        check(&[
            ("a", "apple", true),
            ("z", "apple", false),
            (r"\d", "foo101", true),
            (r"\d", "Art", false),
            (r"\w", "alpha_num3ric", true),
            (r"\w", "$!?", false),
        ]);
    }

    #[test]
    fn groups() {
        check(&[
            ("[abc]", "apple", true),
            ("[abc]", "dog", false),
            ("[^abc]", "cat", true),
            ("[^abc]", "cab", false),
        ]);
    }

    #[test]
    fn sequences() {
        check(&[
            (r"\d apple", "1 apple", true),
            (r"\d apple", "1 orange", false),
            (r"\d\d\d apples", "I got 100 apples", true),
            (r"\d \w\w\ws", "4 cats", true),
            (r"\d \w\w\ws", "1 dog", false),
        ]);
    }

    #[test]
    fn anchors() {
        check(&[
            ("^log", "logs", true),
            ("^log", "slog", false),
            ("dog$", "hotdog", true),
            ("dog$", "dogs", false),
            ("^dog$", "dog", true),
            ("^dog$", "hotdog", false),
            (r"\$5", "cost $5", true),
        ]);
    }

    #[test]
    fn one_or_more() {
        check(&[
            ("a+", "SaaS", true),
            ("ca+ts", "caats", true),
            ("ca+ts", "cts", false),
            (r"\w+s", "cats", true), // needs backtracking
            ("^ca+ts$", "caaats", true),
        ]);
    }

    #[test]
    fn edge_cases() {
        check(&[
            ("", "", true),
            ("a", "", false),
            ("é", "café", true), // multi-byte char
        ]);
    }

    #[test]
    fn zero_or_one() {
        check(&[
            ("dogs?", "dog", true),
            ("dogs?", "dogs", true),
            ("dogs?", "cat", false),
            ("colou?r", "color", true),
            ("colou?r", "colour", true),
            ("colou?r", "colouur", false),
            (r"\d?", "5", true),
            (r"\d?", "", true),
            ("^colou?r$", "colour", true),
            (r"a\?", "a?", true),     // escaped ? is a literal
        ]);
    }
}