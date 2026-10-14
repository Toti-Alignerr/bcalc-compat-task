use bcalc::{run, ErrorKind};

fn out(program: &str) -> String {
    let o = run(program);
    assert_eq!(o.error, None, "unexpected error for {program:?}");
    o.output
}

#[test]
fn arithmetic_and_precedence() {
    assert_eq!(out("1+2*3\n(1+2)*3\n"), "7\n9\n");
}

#[test]
fn assignments_do_not_print() {
    assert_eq!(out("x = 5\nx * 2\n"), "10\n");
}

#[test]
fn comparisons() {
    assert_eq!(out("a = 3\nb = 4\na < b; a == b\n"), "1\n0\n");
}

#[test]
fn division_uses_scale() {
    assert_eq!(out("10 / 3\nscale = 2\n7 / 2\n"), "3\n3.50\n");
}

#[test]
fn integer_powers() {
    assert_eq!(out("2^10\n3^0\n"), "1024\n1\n");
}

#[test]
fn builtin_functions() {
    assert_eq!(out("sqrt(16)\nlength(12345)\nscale(1.250)\n"), "4\n5\n3\n");
}

#[test]
fn increment_and_decrement() {
    assert_eq!(out("x = 1\nx++\nx\n--x\n"), "1\n2\n1\n");
}

#[test]
fn division_by_zero_is_an_error() {
    let o = run("1/0\n");
    assert_eq!(o.output, "");
    assert_eq!(o.error.map(|e| e.kind), Some(ErrorKind::Math));
}
