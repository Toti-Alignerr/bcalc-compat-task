# bcalc

`bcalc` evaluates programs written in the expression subset of the POSIX
`bc` calculator language. The reporting jobs used to shell out to `bc` for
money and ratio arithmetic; they now call `bcalc` in-process, so it must print
exactly what `bc` prints.

Rust 2021, standard library only.

```sh
cargo build
cargo test
printf 'scale=2\n7/2\n' | target/debug/bcalc run
```

## Language

A program is a sequence of statements separated by newlines or `;`.

* **Expression statement:** prints the value followed by a newline.
* **Assignment statement:** `x = expr`, or `+=`, `-=`, `*=`, `/=`, `%=` or `^=`. It prints nothing.

Expressions use:

* numbers such as `12`, `3.25` and `.5`;
* variables (lowercase names such as `x`, `total_2`), which start at 0;
* the special variables `scale`, `ibase`, `obase` and `last`;
* the operators `+ - * / % ^`, unary `-`, `++`/`--` (prefix and postfix), the comparisons `== != < <= > >=` (1 or 0), `!`, `&&` and `||`;
* parentheses;
* the functions `sqrt(e)`, `length(e)` and `scale(e)`.

`#` starts a comment.

`scale` (default 0) is the number of decimal digits kept by division,
`sqrt` and `^`. `ibase` and `obase` are the input and output bases (default 10).

## Library

```rust
let outcome = bcalc::run("scale=2\n7/2\n");
assert_eq!(outcome.output, "3.50\n");
assert!(outcome.error.is_none());
```

* `run(program) -> Outcome`.
* `Outcome { output: String, error: Option<Error> }`, where `output` is everything printed.
* `Error { kind: ErrorKind, message }`, where `kind` is `Parse`, `Math` or `Runtime`.
* `Num`: the number type.

## Command line

```
bcalc run [FILE]
bcalc batch
```

* `run` executes the program in `FILE` (or standard input) and prints its
  output. On an error, the message goes to standard error and the exit code is
  1; otherwise it is 0.
* `batch` reads standard input line by line. Each line is a JSON string
  literal whose value is a complete program (its newlines are written `\n`
  inside the literal). For each input line it prints one line of JSON:
  `{"output":"<everything printed>","error":null}`, or with `"error"` set to
  the error message. An input line that is not a JSON string gets an error
  line with an empty output. Exit code 0.
* Anything else prints usage to standard error and exits with code 2.

```sh
$ printf '%s\n' '"x = 6\nx * 7\n"' '"1/0\n"' | target/debug/bcalc batch
{"output":"42\n","error":null}
{"output":"","error":"Math error: divide by 0"}
```
