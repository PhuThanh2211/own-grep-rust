use crate::ast::{Piece, Token};
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
        let chars: Vec<char> = input.chars().collect();
        let matcher = Matcher { input: &chars, end_anchored: self.end_anchored };

        // `^` means only try position 0; otherwise try every position
        let last_start = if self.start_anchored { 0 } else { chars.len() };

        (0..=last_start).any(|start| {
            matcher.match_seq(start, &self.pieces, &|end| matcher.is_valid_end(end))
        })
    }
}

/// "What must match after this point": receives the position where the
/// current element ended and returns whether the rest of the pattern matches.
type Next<'a> = dyn Fn(usize) -> bool + 'a;

/// State for a single match attempt against one input line.
struct Matcher<'a> {
    input: &'a [char],
    end_anchored: bool,
}

impl Matcher<'_> {
    /// Called once every piece has matched: enforce `$` if present.
    fn is_valid_end(&self, at: usize) -> bool {
        !self.end_anchored || at == self.input.len()
    }

    /// Match `pieces` in order, starting `at`, then continue with `next`.
    fn match_seq(&self, at: usize, pieces: &[Piece], next: &Next) -> bool {
        match pieces.split_first() {
            None => next(at),
            Some((piece, rest)) => {
                self.match_repeat(at, piece, 0, &|end| self.match_seq(end, rest, next))
            }
        }
    }

    fn match_repeat(&self, at: usize, piece: &Piece, count: usize, next: &Next) -> bool {
        let (min, max) = (piece.quant.min, piece.quant.max);
        let can_repeat = max.map_or(true, |max| count < max);

        let matched_more = can_repeat
            && self.match_once(at, &piece.token, &|end| {
            // an empty repetition after `min` would loop forever
            let made_progress = end > at || count < min;
            made_progress && self.match_repeat(end, piece, count + 1, next)
        });

        matched_more || (count >= min && next(at))
    }

    fn match_once(&self, at: usize, token: &Token, next: &Next) -> bool {
        eprintln!("match_once at={at} token={token:?}");

        match token {
            Token::Alt(branches) => branches
                .iter()
                .any(|branch| self.match_seq(at, branch, next)),
            single => self
                .input
                .get(at)
                .is_some_and(|&c| single.matches(c) && next(at + 1)),
        }
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

    #[test]
    fn wildcard() {
        check(&[
            ("d.g", "dog", true),
            ("d.g", "dag", true),
            ("d.g", "d9g", true),
            ("d.g", "cog", false),
            ("d.g", "dg", false),
            ("...", "cat", true),
            ("...", "ca", false),
            (r".\d.", "a1b", true),
            ("g.+gol", "goøö0Ogol", true), // . with + and non-ASCII
            (r"a\.b", "a.b", true),        // escaped dot is literal
            (r"a\.b", "axb", false),
            ("a[.]b", "axb", false),       // dot in group is literal
        ]);
    }

    #[test]
    fn alternation() {
        check(&[
            // ("(cat|dog)", "cat", true),
            ("(cat|dog)", "dog", true),
            // ("(cat|dog)", "apple", false),
            // ("(cat|dog)", "doghouse", true),
            // ("I like (cats|dogs)", "I like cats", true),
            // ("I like (cats|dogs)", "I like dogs", true),
            // ("(red|blue|green)", "blue", true),
            // ("^(a|ab)c$", "abc", true),        // needs backtracking into branch
            // ("^(cat|dog)s?$", "dogs", true),
            // ("(ab)+c", "ababc", true),         // quantified group
            // ("cat|dog", "hotdog", true),       // top-level alternation
            // (r"a\(b", "a(b", true),            // escaped paren
        ]);
    }
}