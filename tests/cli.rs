use std::io::Write;
use std::process::{Command, Stdio};

fn run(pattern: &str, input: &str) -> (i32, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codecrafters-grep"))
        .args(["-E", pattern])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn prints_matching_line() {
    assert_eq!(run("[^abc]", "apple"), (0, "apple\n".to_string()));
    assert_eq!(run("dog$", "dog"), (0, "dog\n".to_string()));
}

#[test]
fn prints_nothing_on_no_match() {
    assert_eq!(run("[^abc]", "cab"), (1, String::new()));
}