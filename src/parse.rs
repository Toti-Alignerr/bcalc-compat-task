//! Lexer and recursive-descent parser for one line of bc.

use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Num(String),
    Name(String),
    Op(&'static str),
    LParen,
    RParen,
    Semi,
    End,
    Bad(char),
}

const OPS: [&str; 24] = [
    "++", "--", "+=", "-=", "*=", "/=", "%=", "^=", "==", "!=", "<=", ">=", "&&", "||", "+", "-", "*", "/", "%", "^",
    "=", "<", ">", "!",
];

pub fn tokenize(line: &str) -> Vec<Tok> {
    let b = line.as_bytes();
    let mut i = 0;
    let mut toks = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c == b' ' || c == b'\t' {
            i += 1;
            continue;
        }
        if c == b'#' {
            break;
        }
        if c.is_ascii_digit() || c == b'.' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            toks.push(Tok::Num(line[start..i].to_string()));
            continue;
        }
        if c.is_ascii_lowercase() {
            let start = i;
            while i < b.len() && (b[i].is_ascii_lowercase() || b[i].is_ascii_digit() || b[i] == b'_') {
                i += 1;
            }
            toks.push(Tok::Name(line[start..i].to_string()));
            continue;
        }
        match c {
            b'(' => toks.push(Tok::LParen),
            b')' => toks.push(Tok::RParen),
            b';' => toks.push(Tok::Semi),
            _ => {
                if let Some(op) = OPS.iter().find(|op| line[i..].starts_with(**op)) {
                    toks.push(Tok::Op(op));
                    i += op.len();
                    continue;
                }
                toks.push(Tok::Bad(line[i..].chars().next().unwrap()));
                i += line[i..].chars().next().unwrap().len_utf8();
                continue;
            }
        }
        i += 1;
    }
    toks.push(Tok::End);
    toks
}

#[derive(Clone, Debug)]
pub enum Expr {
    Num(String),
    Var(String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    PreInc(String, bool), // true = increment
    PostInc(String, bool),
    Bin(&'static str, Box<Expr>, Box<Expr>),
    Assign(&'static str, String, Box<Expr>),
    Call(String, Box<Expr>),
}

pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    pub fn new(toks: Vec<Tok>) -> Parser {
        Parser { toks, pos: 0 }
    }

    pub fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }

    pub fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn is_op(&self, op: &str) -> bool {
        matches!(self.peek(), Tok::Op(o) if *o == op)
    }

    fn bad(&self) -> Error {
        match self.peek() {
            Tok::Bad(c) => Error::parse(&format!("bad character '{}'", c)),
            _ => Error::parse("bad expression"),
        }
    }

    pub fn expr(&mut self) -> Result<Expr, Error> {
        let l = self.or()?;
        let op = match self.peek() {
            Tok::Op(o) if matches!(*o, "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "^=") => *o,
            _ => return Ok(l),
        };
        let name = match l {
            Expr::Var(n) => n,
            _ => return Err(Error::parse("bad assignment")),
        };
        self.advance();
        let r = self.expr()?;
        Ok(Expr::Assign(op, name, Box::new(r)))
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let mut l = self.and()?;
        while self.is_op("||") {
            self.advance();
            let r = self.and()?;
            l = Expr::Bin("||", Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let mut l = self.rel()?;
        while self.is_op("&&") {
            self.advance();
            let r = self.rel()?;
            l = Expr::Bin("&&", Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn rel(&mut self) -> Result<Expr, Error> {
        let l = self.add()?;
        let op = match self.peek() {
            Tok::Op(o) if matches!(*o, "==" | "!=" | "<" | "<=" | ">" | ">=") => *o,
            _ => return Ok(l),
        };
        self.advance();
        let r = self.add()?;
        Ok(Expr::Bin(op, Box::new(l), Box::new(r)))
    }

    fn add(&mut self) -> Result<Expr, Error> {
        let mut l = self.mul()?;
        loop {
            let op = match self.peek() {
                Tok::Op(o) if matches!(*o, "+" | "-") => *o,
                _ => return Ok(l),
            };
            self.advance();
            let r = self.mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
    }

    fn mul(&mut self) -> Result<Expr, Error> {
        let mut l = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Op(o) if matches!(*o, "*" | "/" | "%") => *o,
                _ => return Ok(l),
            };
            self.advance();
            let r = self.unary()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_op("-") {
            self.advance();
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.is_op("!") {
            self.advance();
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        if self.is_op("++") || self.is_op("--") {
            let inc = self.is_op("++");
            self.advance();
            return match self.advance() {
                Tok::Name(n) => Ok(Expr::PreInc(n, inc)),
                _ => Err(Error::parse("bad assignment")),
            };
        }
        self.pow()
    }

    fn pow(&mut self) -> Result<Expr, Error> {
        let base = self.postfix()?;
        if self.is_op("^") {
            self.advance();
            let e = self.unary()?;
            return Ok(Expr::Bin("^", Box::new(base), Box::new(e)));
        }
        Ok(base)
    }

    fn postfix(&mut self) -> Result<Expr, Error> {
        let p = self.primary()?;
        if self.is_op("++") || self.is_op("--") {
            let inc = self.is_op("++");
            if let Expr::Var(n) = p {
                self.advance();
                return Ok(Expr::PostInc(n, inc));
            }
            return Err(Error::parse("bad assignment"));
        }
        Ok(p)
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        match self.peek().clone() {
            Tok::Num(s) => {
                if s.matches('.').count() > 1 || s == "." {
                    return Err(self.bad());
                }
                self.advance();
                Ok(Expr::Num(s))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expr()?;
                if *self.peek() != Tok::RParen {
                    return Err(self.bad());
                }
                self.advance();
                Ok(e)
            }
            Tok::Name(n) => {
                self.advance();
                if matches!(n.as_str(), "sqrt" | "length" | "scale") && *self.peek() == Tok::LParen {
                    self.advance();
                    let e = self.expr()?;
                    if *self.peek() != Tok::RParen {
                        return Err(self.bad());
                    }
                    self.advance();
                    return Ok(Expr::Call(n, Box::new(e)));
                }
                if matches!(n.as_str(), "sqrt" | "length") {
                    return Err(self.bad());
                }
                Ok(Expr::Var(n))
            }
            _ => Err(self.bad()),
        }
    }
}
