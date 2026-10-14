use std::io::Write;
use std::process::{Command, Stdio};

fn bcalc(args: &[&str], input: &str) -> (i32, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bcalc"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let o = child.wait_with_output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8(o.stdout).unwrap())
}

#[test]
fn run_prints_program_output() {
    assert_eq!(bcalc(&["run"], "3*4\n"), (0, "12\n".to_string()));
}

#[test]
fn run_exits_1_on_error() {
    assert_eq!(bcalc(&["run"], "1/0\n").0, 1);
}

#[test]
fn batch_answers_one_json_line_per_program() {
    let (code, out) = bcalc(&["batch"], "\"1+1\\n\"\nnot json\n");
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "{\"output\":\"2\\n\",\"error\":null}");
    assert!(lines[1].starts_with("{\"output\":\"\",\"error\":\""));
}
