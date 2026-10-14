use bcalc::{json, run, Outcome};
use std::io::{self, BufRead, Read, Write};
use std::process::ExitCode;

fn result_line(o: &Outcome) -> String {
    let err = match &o.error {
        Some(e) => json::quote(&e.to_string()),
        None => "null".to_string(),
    };
    format!("{{\"output\":{},\"error\":{}}}", json::quote(&o.output), err)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") if args.len() <= 2 => {
            let mut program = String::new();
            let read = match args.get(1) {
                Some(path) => std::fs::read_to_string(path).map(|s| program = s),
                None => io::stdin().read_to_string(&mut program).map(|_| ()),
            };
            if let Err(e) = read {
                eprintln!("bcalc: {}", e);
                return ExitCode::from(1);
            }
            let o = run(&program);
            print!("{}", o.output);
            match o.error {
                Some(e) => {
                    eprintln!("{}", e);
                    ExitCode::from(1)
                }
                None => ExitCode::SUCCESS,
            }
        }
        Some("batch") if args.len() == 1 => {
            let stdin = io::stdin().lock();
            let mut stdout = io::BufWriter::new(io::stdout().lock());
            for line in stdin.lines() {
                let line = line.unwrap_or_default();
                let res = match json::parse_string_literal(&line) {
                    Some(program) => result_line(&run(&program)),
                    None => "{\"output\":\"\",\"error\":\"input line is not a JSON string\"}".to_string(),
                };
                writeln!(stdout, "{}", res).unwrap();
            }
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("usage: bcalc run [FILE]\n       bcalc batch");
            ExitCode::from(2)
        }
    }
}
