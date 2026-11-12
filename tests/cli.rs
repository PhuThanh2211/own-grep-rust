use std::io::Write;
use std::process::{Command, Stdio};

fn run(pattern: &str, input: &str) -> i32 {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codecrafters-grep"))
        .args(["-E", pattern])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait().unwrap().code().unwrap()
}

#[test]
fn exit_codes() {
    assert_eq!(run(r"\d", "apple123"), 0);
    assert_eq!(run(r"\d", "apple"), 1);
}

#[test]
fn trailing_newline_is_ignored() {
    assert_eq!(run("dog$", "hotdog\n"), 0);
    assert_eq!(run("dog$", "hotdog\r\n"), 0);
    assert_eq!(run("[^abc]", "cab\n"), 1);
}

#[test]
fn invalid_usage_exits_2() {
    assert_eq!(run("[abc", "a"), 2);
}