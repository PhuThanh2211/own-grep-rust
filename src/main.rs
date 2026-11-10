use std::env;
use std::io;
use std::process;

enum Token {
    Literal(char),
    Digit,          // \d
    Word,           // \w
    Group {         // [abc], [^abc]
        chars: Vec<char>,
        negated: bool
    }
}

impl Token {
    fn matches(&self, c: char) -> bool {
        match self {
            Token::Literal(l) => *l == c,
            Token::Digit => c.is_ascii_digit(),
            Token::Word => c.is_ascii_alphanumeric() || c == '_',
            Token::Group { chars, negated} => chars.contains(&c) != *negated,
        }
    }
}

fn parse(pattern: &str) -> Vec<Token> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                let next = *chars.get(i + 1).expect("trailing backslash");
                tokens.push(match next {
                    'd' => Token::Digit,
                    'w' => Token::Word,
                    other => Token::Literal(other), // e.g. \\ or \[
                });
                i += 2;
            }
            '[' => {
                let end = i + chars[i..].iter().position(|&c| c == ']').expect("unclosed [");
                let mut inner = &chars[i + 1..end];
                let negated = inner.first() == Some(&'^');
                if negated { inner = &inner[1..]; }
                tokens.push(Token::Group { chars: inner.to_vec(), negated });
                i = end + 1;
            }
            c => { tokens.push(Token::Literal(c)); i += 1; }
        }
    }
    tokens
}
fn match_here(input: &[char], tokens: &[Token], end_anchored: bool) -> bool {
    match tokens.split_first() {
        None => !end_anchored || input.is_empty(),
        Some((tok, rest)) => match input.split_first() {
            Some((&c, input_rest)) if tok.matches(c) => match_here(input_rest, rest, end_anchored),
            _ => false,
        },
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let input: Vec<char> = input_line.chars().collect();

    let (start_anchored, pattern) = match pattern.strip_prefix('^') {
        Some(p) => (true, p),
        None => (false, pattern)
    };

    let (end_anchored, pattern) = match pattern.strip_suffix('$') {
        Some(p) if !p.ends_with('\\') => (true, p), // `\$` is a literal dollar
        _ => (false, pattern)
    };

    let tokens = parse(pattern);

    if start_anchored {
        match_here(&input, &tokens, end_anchored)
    } else {
        (0..=input.len()).any(|start| match_here(&input[start..], &tokens, end_anchored))
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();
    let input_line = input_line.trim_end_matches(['\n', '\r']);

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digit_class() {
        assert!(match_pattern("foo101", r"\d"));
        assert!(!match_pattern("Art", r"\d"));
    }

    #[test]
    fn word_class() {
        assert!(match_pattern("alpha_num3ric", r"\w"));
        assert!(match_pattern("_", r"\w"));
        assert!(!match_pattern("$!?", r"\w"));
    }

    #[test]
    fn positive_group() {
        assert!(match_pattern("apple", "[abc]"));
        assert!(match_pattern("cab", "[abc]"));
        assert!(!match_pattern("dog", "[abc]"));
        assert!(match_pattern("a1b2c3", "[123]"));
    }

    #[test]
    fn negative_group() {
        assert!(match_pattern("cat", "[^abc]"));
        assert!(!match_pattern("cab", "[^abc]"));
        assert!(match_pattern("apple", "[^abc]"));
    }

    #[test]
    fn sequences() {
        assert!(match_pattern("1 apple", r"\d apple"));
        assert!(!match_pattern("1 orange", r"\d apple"));
        assert!(match_pattern("I got 100 apples from the store", r"\d\d\d apples"));
        assert!(!match_pattern("I got 1 apple from the store", r"\d\d\d apples"));
        assert!(match_pattern("4 cats", r"\d \w\w\ws"));
        assert!(!match_pattern("1 dog", r"\d \w\w\ws"));
    }

    #[test]
    fn start_anchor() {
        assert!(match_pattern("log", "^log"));
        assert!(match_pattern("logs", "^log"));
        assert!(!match_pattern("slog", "^log"));
        assert!(match_pattern("123abc", r"^\d\d\d"));
        assert!(match_pattern("cat", "[^abc]"));
    }

    #[test]
    fn end_anchor() {
        assert!(match_pattern("dog", "dog$"));
        assert!(match_pattern("hotdog", "dog$"));
        assert!(!match_pattern("dogs", "dog$"));
        assert!(match_pattern("abc123", r"\d\d\d$"));
        assert!(!match_pattern("abc123@", r"\w\w\w$"));
        assert!(match_pattern("abc123cde", r"\w\w\w$"));
        assert!(match_pattern("dog", "^dog$"));
        assert!(!match_pattern("hotdog", "^dog$"));
    }
}