#![forbid(unsafe_code)]
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { r: u32 },
    Rect(u32, u32),
}

struct Point {
    x: i64,
    y: i64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug)]
enum ParseError {
    Empty,
    Bad(String),
}

fn parse(s: &str) -> Result<u32, ParseError> {
    let t = s.trim();
    if t.is_empty() {
        return Err(ParseError::Empty);
    }
    t.parse::<u32>().map_err(|e| ParseError::Bad(e.to_string()))
}

fn sum_all(items: &[&str]) -> Result<u32, ParseError> {
    let mut total = 0u32;
    for s in items {
        let v = parse(s)?;
        total = total.checked_add(v).ok_or(ParseError::Bad("overflow".to_string()))?;
    }
    Ok(total)
}

fn area(s: &Shape) -> u32 {
    match s {
        Shape::Circle { r } => r.wrapping_mul(*r).wrapping_mul(3),
        Shape::Rect(w, h) => w.saturating_mul(*h),
    }
}

pub fn run() -> String {
    let mut v: Vec<u32> = (1..=10).filter(|n| n % 2 == 0).map(|n| n * n).collect();
    v.push(7);
    v.sort();
    v.reverse();
    let total: u32 = v.iter().copied().sum();
    let max = v.iter().max().copied().unwrap_or(0);
    let words: Vec<String> = "a b  c".split_whitespace().map(|w| w.to_uppercase()).collect();
    let joined = words.join("-");
    let mut map = BTreeMap::new();
    for (i, w) in words.iter().enumerate() {
        map.insert(w.clone(), i);
    }
    let first = map.get("A").copied();
    let shared = Rc::new(RefCell::new(Vec::<i32>::new()));
    let s2 = Rc::clone(&shared);
    s2.borrow_mut().push(5);
    let boxed: Box<dyn Fn(u32) -> u32> = Box::new(move |x| x + total);
    let shapes = vec![Shape::Circle { r: 2 }, Shape::Rect(3, 4)];
    let areas: Vec<u32> = shapes.iter().map(area).collect();
    let slice = &v[1..3];
    let opt: Option<&u32> = slice.first();
    let p = Point { x: -1, y: 2 };
    assert!(matches!(shapes[0], Shape::Circle { .. }));
    assert_eq!(shapes[1].clone(), Shape::Rect(3, 4));
    let n = 250u8.checked_add(10);
    let w = i32::MAX.wrapping_add(1);
    let r = sum_all(&["1", " 2 ", "3"]);
    let bad = sum_all(&["", "x"]);
    let mut s = String::new();
    s.push_str(&joined);
    s += "!";
    let chars: usize = s.chars().count();
    let starts = s.starts_with("A");
    format!(
        "{v:?} {total} {max} {joined} {first:?} {:?} {} {areas:?} {opt:?} {p} {n:?} {w} {r:?} {bad:?} {chars} {starts} {}",
        shared.borrow(),
        boxed(1),
        s.len(),
    )
}

fn main() {
    let out = run();
    assert!(!out.is_empty());
}
