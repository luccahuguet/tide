use std::{
    fmt,
    ops::{Add, Mul, Neg, Sub},
};
#[derive(Debug, Clone, PartialEq)]
pub struct Poly(Vec<i64>);
impl Poly {
    pub fn new(mut coeffs: Vec<i64>) -> Self {
        while coeffs.last() == Some(&0) {
            coeffs.pop();
        }
        Self(coeffs)
    }
    pub fn coeffs(&self) -> &[i64] {
        &self.0
    }
    pub fn degree(&self) -> Option<usize> {
        self.0.len().checked_sub(1)
    }
    pub fn eval(&self, x: i64) -> Option<i64> {
        self.0
            .iter()
            .rev()
            .try_fold(0i64, |n, c| n.checked_mul(x)?.checked_add(*c))
    }
}
impl Add for &Poly {
    type Output = Poly;
    fn add(self, b: Self) -> Poly {
        Poly::new(
            (0..self.0.len().max(b.0.len()))
                .map(|i| self.0.get(i).unwrap_or(&0) + b.0.get(i).unwrap_or(&0))
                .collect(),
        )
    }
}
impl Sub for &Poly {
    type Output = Poly;
    fn sub(self, b: Self) -> Poly {
        Poly::new(
            (0..self.0.len().max(b.0.len()))
                .map(|i| self.0.get(i).unwrap_or(&0) - b.0.get(i).unwrap_or(&0))
                .collect(),
        )
    }
}
impl Mul for &Poly {
    type Output = Poly;
    fn mul(self, b: Self) -> Poly {
        if self.0.is_empty() || b.0.is_empty() {
            return Poly::new(vec![]);
        }
        let mut c = vec![0; self.0.len() + b.0.len() - 1];
        for (i, a) in self.0.iter().enumerate() {
            for (j, b) in b.0.iter().enumerate() {
                c[i + j] += a * b;
            }
        }
        Poly::new(c)
    }
}
impl Add for Poly {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        &self + &b
    }
}
impl Sub for Poly {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        &self - &b
    }
}
impl Mul for Poly {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        &self * &b
    }
}
impl Neg for Poly {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(self.0.into_iter().map(|c| -c).collect())
    }
}
impl fmt::Display for Poly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (i, &c) in self.0.iter().enumerate().rev().filter(|(_, c)| **c != 0) {
            if first {
                if c < 0 {
                    write!(f, "-")?;
                }
            } else {
                write!(f, " {} ", if c < 0 { "-" } else { "+" })?;
            }
            let mag = i128::from(c).abs();
            if i == 0 || mag != 1 {
                write!(f, "{mag}")?;
            }
            if i > 0 {
                write!(f, "x")?;
            }
            if i > 1 {
                write!(f, "^{i}")?;
            }
            first = false;
        }
        if first {
            write!(f, "0")?;
        }
        Ok(())
    }
}
