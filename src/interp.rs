//! Executes a program line by line and collects what bc would print.

use crate::num::Num;
use crate::parse::{tokenize, Expr, Parser, Tok};
use crate::{Error, Outcome};
use std::cmp::Ordering;
use std::collections::HashMap;

struct Env {
    vars: HashMap<String, Num>,
    scale: u32,
    ibase: u32,
    obase: u32,
    last: Num,
    out: String,
}

/// Run a whole program. A failing statement is reported and skipped; the
/// first error is returned in the outcome.
pub fn run(program: &str) -> Outcome {
    let mut env = Env { vars: HashMap::new(), scale: 0, ibase: 10, obase: 10, last: Num::zero(), out: String::new() };
    let mut first_error = None;
    for line in program.lines() {
        let mut p = Parser::new(tokenize(line));
        loop {
            match p.peek() {
                Tok::End => break,
                Tok::Semi => {
                    p.advance();
                    continue;
                }
                _ => {}
            }
            let stmt = match p.expr() {
                Ok(e) => e,
                Err(e) => {
                    first_error.get_or_insert(e);
                    break; // skip the rest of the line
                }
            };
            if !matches!(p.peek(), Tok::Semi | Tok::End) {
                first_error.get_or_insert(Error::parse("bad expression"));
                break;
            }
            match env.eval(&stmt) {
                Ok(v) => {
                    if !matches!(stmt, Expr::Assign(..)) {
                        env.out.push_str(&v.to_base_string(env.obase));
                        env.out.push('\n');
                        env.last = v;
                    }
                }
                Err(e) => {
                    first_error.get_or_insert(e);
                }
            }
        }
    }
    Outcome { output: env.out, error: first_error }
}

fn bool_num(b: bool) -> Num {
    Num::from_i64(b as i64)
}

impl Env {
    fn load(&self, name: &str) -> Num {
        match name {
            "scale" => Num::from_i64(self.scale as i64),
            "ibase" => Num::from_i64(self.ibase as i64),
            "obase" => Num::from_i64(self.obase as i64),
            "last" => self.last,
            _ => self.vars.get(name).copied().unwrap_or_else(Num::zero),
        }
    }

    fn store(&mut self, name: &str, v: Num) -> Result<(), Error> {
        let int = || v.to_i64_trunc().filter(|i| *i >= 0).ok_or_else(|| Error::math("negative number"));
        match name {
            "scale" => self.scale = int()? as u32,
            "ibase" => match int()? {
                i @ 2..=16 => self.ibase = i as u32,
                _ => return Err(Error::runtime("bad ibase")),
            },
            "obase" => match int()? {
                i @ 2..=16 => self.obase = i as u32,
                _ => return Err(Error::runtime("bad obase")),
            },
            "last" => self.last = v,
            _ => {
                self.vars.insert(name.to_string(), v);
            }
        }
        Ok(())
    }

    fn binary(&self, op: &str, a: Num, b: Num) -> Result<Num, Error> {
        let s = self.scale;
        Ok(match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b, s),
            "/" => a.div(&b, s)?,
            "%" => a.rem(&b, s)?,
            "^" => a.pow(&b, s)?,
            "==" => bool_num(a.cmp(&b) == Ordering::Equal),
            "!=" => bool_num(a.cmp(&b) != Ordering::Equal),
            "<" => bool_num(a.cmp(&b) == Ordering::Less),
            "<=" => bool_num(a.cmp(&b) != Ordering::Greater),
            ">" => bool_num(a.cmp(&b) == Ordering::Greater),
            ">=" => bool_num(a.cmp(&b) != Ordering::Less),
            _ => unreachable!("operator {op}"),
        })
    }

    fn eval(&mut self, e: &Expr) -> Result<Num, Error> {
        match e {
            Expr::Num(text) => Ok(Num::parse_literal(text)),
            Expr::Var(n) => Ok(self.load(n)),
            Expr::Neg(inner) => Ok(self.eval(inner)?.neg()),
            Expr::Not(inner) => Ok(bool_num(self.eval(inner)?.is_zero())),
            Expr::PreInc(n, inc) => self.step(n, *inc),
            Expr::PostInc(n, inc) => {
                let old = self.load(n);
                self.step(n, *inc)?;
                Ok(old)
            }
            Expr::Bin("&&", l, r) => {
                if self.eval(l)?.is_zero() {
                    return Ok(bool_num(false));
                }
                Ok(bool_num(!self.eval(r)?.is_zero()))
            }
            Expr::Bin("||", l, r) => {
                if !self.eval(l)?.is_zero() {
                    return Ok(bool_num(true));
                }
                Ok(bool_num(!self.eval(r)?.is_zero()))
            }
            Expr::Bin(op, l, r) => {
                let a = self.eval(l)?;
                let b = self.eval(r)?;
                self.binary(op, a, b)
            }
            Expr::Assign(op, name, rhs) => {
                let v = if *op == "=" {
                    self.eval(rhs)?
                } else {
                    let cur = self.load(name);
                    let r = self.eval(rhs)?;
                    self.binary(&op[..1], cur, r)?
                };
                self.store(name, v)?;
                Ok(v)
            }
            Expr::Call(name, arg) => {
                let v = self.eval(arg)?;
                match name.as_str() {
                    "sqrt" => v.sqrt(self.scale),
                    "length" => Ok(Num::from_i64(v.length() as i64)),
                    _ => Ok(Num::from_i64(v.scale() as i64)),
                }
            }
        }
    }

    fn step(&mut self, n: &str, inc: bool) -> Result<Num, Error> {
        let one = Num::from_i64(1);
        let cur = self.load(n);
        let v = if inc { cur.add(&one) } else { cur.sub(&one) };
        self.store(n, v)?;
        Ok(v)
    }
}
