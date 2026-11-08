use std::fmt;
#[derive(Debug, PartialEq)]
pub enum CalcError {
    DivByZero { pos: usize },
    Overflow { pos: usize },
    Unexpected { pos: usize },
}
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivByZero { pos } => write!(f, "division by zero at {pos}"),
            Self::Overflow { pos } => write!(f, "overflow at {pos}"),
            Self::Unexpected { pos } => write!(f, "unexpected input at {pos}"),
        }
    }
}
struct Parser<'a> {
    src: &'a [u8],
    at: usize,
}
impl Parser<'_> {
    fn peek(&mut self) -> Option<u8> {
        while self.src.get(self.at) == Some(&b' ') {
            self.at += 1;
        }
        self.src.get(self.at).copied()
    }
    fn atom(&mut self) -> Result<i64, CalcError> {
        let c = self.peek();
        let pos = self.at;
        match c {
            Some(b'-') => {
                self.at += 1;
                self.atom()?
                    .checked_neg()
                    .ok_or(CalcError::Overflow { pos })
            }
            Some(b'(') => {
                self.at += 1;
                let n = self.expr(0)?;
                if self.peek() != Some(b')') {
                    return Err(CalcError::Unexpected { pos: self.at });
                }
                self.at += 1;
                Ok(n)
            }
            Some(b'0'..=b'9') => {
                let mut n = 0i64;
                while let Some(&b) = self.src.get(self.at).filter(|b| b.is_ascii_digit()) {
                    n = n
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(i64::from(b - b'0')))
                        .ok_or(CalcError::Overflow { pos })?;
                    self.at += 1;
                }
                Ok(n)
            }
            _ => Err(CalcError::Unexpected { pos }),
        }
    }
    fn expr(&mut self, min: u8) -> Result<i64, CalcError> {
        let mut n = self.atom()?;
        loop {
            let op = self.peek();
            let priority = match op {
                Some(b'+' | b'-') => 1,
                Some(b'*' | b'/' | b'%') => 2,
                _ => 0,
            };
            if priority <= min {
                return Ok(n);
            }
            let pos = self.at;
            self.at += 1;
            let rhs = self.expr(priority)?;
            if matches!(op, Some(b'/' | b'%')) && rhs == 0 {
                return Err(CalcError::DivByZero { pos });
            }
            n = match op.unwrap() {
                b'+' => n.checked_add(rhs),
                b'-' => n.checked_sub(rhs),
                b'*' => n.checked_mul(rhs),
                b'/' => n.checked_div(rhs),
                b'%' => n.checked_rem(rhs),
                _ => unreachable!(),
            }
            .ok_or(CalcError::Overflow { pos })?;
        }
    }
}
pub fn eval(src: &str) -> Result<i64, CalcError> {
    let mut p = Parser {
        src: src.as_bytes(),
        at: 0,
    };
    let n = p.expr(0)?;
    if p.peek().is_some() {
        Err(CalcError::Unexpected { pos: p.at })
    } else {
        Ok(n)
    }
}
