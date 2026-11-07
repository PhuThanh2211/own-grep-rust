use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == r"\d" {
        input_line.chars().any(|c| is_digit(c))
    } else if pattern == r"\w" {
        input_line.chars().any(|c| is_word(c))
    } else if pattern.starts_with('[') && pattern.ends_with(']') && pattern.len() > 2 {
        let group = &pattern[1..pattern.len() - 1];
        input_line.chars().any(|c| group.contains(c))
    } else if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
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
}