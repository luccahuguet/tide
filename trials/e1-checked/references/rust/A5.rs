pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> &str;
}
pub struct Circle(f64);
pub struct Rect(f64, f64);
pub struct Triangle(f64, f64, f64);
fn valid(x: f64) -> bool {
    x.is_finite() && x > 0.0
}
impl Circle {
    pub fn new(r: f64) -> Option<Self> {
        valid(r).then_some(Self(r))
    }
}
impl Rect {
    pub fn new(w: f64, h: f64) -> Option<Self> {
        (valid(w) && valid(h)).then_some(Self(w, h))
    }
}
impl Triangle {
    pub fn new(a: f64, b: f64, c: f64) -> Option<Self> {
        (valid(a) && valid(b) && valid(c) && a + b > c && a + c > b && b + c > a)
            .then_some(Self(a, b, c))
    }
}
impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.0 * self.0
    }
    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.0
    }
    fn name(&self) -> &str {
        "circle"
    }
}
impl Shape for Rect {
    fn area(&self) -> f64 {
        self.0 * self.1
    }
    fn perimeter(&self) -> f64 {
        2.0 * (self.0 + self.1)
    }
    fn name(&self) -> &str {
        "rect"
    }
}
impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = self.perimeter() / 2.0;
        (s * (s - self.0) * (s - self.1) * (s - self.2)).sqrt()
    }
    fn perimeter(&self) -> f64 {
        self.0 + self.1 + self.2
    }
    fn name(&self) -> &str {
        "triangle"
    }
}
pub fn largest(shapes: &[Box<dyn Shape>]) -> Option<&dyn Shape> {
    let mut best: Option<&dyn Shape> = None;
    for s in shapes {
        if best.is_none_or(|b| s.area() > b.area()) {
            best = Some(&**s);
        }
    }
    best
}
pub fn sort_by_area(shapes: &mut [Box<dyn Shape>]) {
    shapes.sort_by(|a, b| a.area().total_cmp(&b.area()));
}
pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}
pub fn describe(s: &dyn Shape) -> String {
    format!(
        "{}: area {:.2}, perimeter {:.2}",
        s.name(),
        s.area(),
        s.perimeter()
    )
}
