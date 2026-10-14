//! Fixed-point decimal numbers: a 128-bit integer mantissa and a scale
//! (number of digits after the decimal point). 38 significant digits are
//! plenty for the reporting jobs.

use crate::Error;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Num {
    v: i128, // value = v / 10^scale
    scale: u32,
}

fn pow10(n: u32) -> i128 {
    10i128.pow(n)
}

impl Num {
    pub fn zero() -> Num {
        Num { v: 0, scale: 0 }
    }

    pub fn from_i64(v: i64) -> Num {
        Num { v: v as i128, scale: 0 }
    }

    pub fn scale(&self) -> usize {
        self.scale as usize
    }

    pub fn is_zero(&self) -> bool {
        self.v == 0
    }

    pub fn is_negative(&self) -> bool {
        self.v < 0
    }

    /// Parses a decimal literal such as `12`, `3.25` or `.5`.
    pub fn parse_literal(text: &str) -> Num {
        let mut v: i128 = 0;
        let mut scale = 0;
        let mut frac = false;
        for c in text.chars() {
            if c == '.' {
                frac = true;
                continue;
            }
            v = v * 10 + c.to_digit(10).unwrap() as i128;
            if frac {
                scale += 1;
            }
        }
        Num { v, scale }
    }

    fn rescale(&self, s: u32) -> i128 {
        self.v * pow10(s - self.scale)
    }

    /// Truncate to at most `s` fractional digits.
    pub fn truncate(&self, s: u32) -> Num {
        if s >= self.scale {
            return *self;
        }
        Num { v: self.v / pow10(self.scale - s), scale: s }
    }

    pub fn neg(&self) -> Num {
        Num { v: -self.v, scale: self.scale }
    }

    pub fn cmp(&self, other: &Num) -> Ordering {
        let s = self.scale.max(other.scale);
        self.rescale(s).cmp(&other.rescale(s))
    }

    pub fn add(&self, other: &Num) -> Num {
        let s = self.scale.max(other.scale);
        Num { v: self.rescale(s) + other.rescale(s), scale: s }
    }

    pub fn sub(&self, other: &Num) -> Num {
        self.add(&other.neg())
    }

    pub fn mul(&self, other: &Num, _scale: u32) -> Num {
        Num { v: self.v * other.v, scale: self.scale + other.scale }
    }

    pub fn div(&self, other: &Num, scale: u32) -> Result<Num, Error> {
        if other.v == 0 {
            return Err(Error::math("divide by 0"));
        }
        let s = self.scale.max(other.scale);
        let num = self.rescale(s) * pow10(scale);
        Ok(Num { v: num / other.rescale(s), scale })
    }

    pub fn rem(&self, other: &Num, _scale: u32) -> Result<Num, Error> {
        if other.v == 0 {
            return Err(Error::math("divide by 0"));
        }
        let s = self.scale.max(other.scale);
        Ok(Num { v: self.rescale(s) % other.rescale(s), scale: s })
    }

    pub fn pow(&self, exp: &Num, scale: u32) -> Result<Num, Error> {
        if exp.scale != 0 && exp.v % pow10(exp.scale) != 0 {
            return Err(Error::math("non-integer number"));
        }
        let e = exp.truncate(0).v;
        if e < 0 {
            return Err(Error::math("negative exponent"));
        }
        let mut acc = Num::from_i64(1);
        for _ in 0..e {
            acc = acc.mul(self, scale);
        }
        Ok(acc.truncate(scale))
    }

    pub fn sqrt(&self, scale: u32) -> Result<Num, Error> {
        if self.v < 0 {
            return Err(Error::math("negative number"));
        }
        let rs = scale.max(self.scale);
        let f = (self.v as f64 / pow10(self.scale) as f64).sqrt();
        Ok(Num { v: (f * pow10(rs) as f64) as i128, scale: rs })
    }

    /// bc's length(): number of significant decimal digits.
    pub fn length(&self) -> usize {
        if self.v == 0 {
            return (self.scale as usize).max(1);
        }
        self.v.unsigned_abs().to_string().len()
    }

    pub fn to_i64_trunc(&self) -> Option<i64> {
        i64::try_from(self.truncate(0).v).ok()
    }

    pub fn to_string(&self) -> String {
        if self.scale == 0 {
            return self.v.to_string();
        }
        let digits = format!("{:0w$}", self.v.unsigned_abs(), w = self.scale as usize + 1);
        let (i, f) = digits.split_at(digits.len() - self.scale as usize);
        format!("{}{}.{}", if self.v < 0 { "-" } else { "" }, i, f)
    }

    /// Integer part in another output base (2-16).
    pub fn to_base_string(&self, obase: u32) -> String {
        if obase == 10 || self.scale != 0 {
            return self.to_string();
        }
        let mut n = self.v.unsigned_abs();
        if n == 0 {
            return "0".to_string();
        }
        let mut digits = Vec::new();
        while n > 0 {
            digits.push(std::char::from_digit((n % obase as u128) as u32, obase).unwrap().to_ascii_uppercase());
            n /= obase as u128;
        }
        let s: String = digits.iter().rev().collect();
        if self.v < 0 { format!("-{}", s) } else { s }
    }
}
