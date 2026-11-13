//@ rules: CLAUSE-1, CLAUSE-6, FORM-4
// Independent generated-test runner fixture; build.rs executes entries individually.
#![deny(unused_imports, unused_braces)]
#[ensures(result >= n)]
fn rebound(n: u8) -> u8 { if n == u8::MAX { 0 } else { n + 1 } }

#[requires(i < values.len())]
#[ensures(result == values[i])]
fn select(values: &[u16], i: usize) -> u16 { values[i] }

#[ensures(result == n)]
fn identity<T: Copy + PartialEq + std::marker::Pure + Sync>(n: T) -> T { n }

#[ensures(result == n.as_ref().len())]
fn string_bound<T: AsRef<str> + std::marker::Pure + Sync>(n: T) -> usize { n.as_ref().len() }

trait Missing {}
#[ensures(result)]
fn unmet<T: Missing>(_: T) -> bool { true }

#[ensures(result)]
fn closure_arg(f: impl Fn() -> bool + std::marker::Pure + Sync) -> bool { f() }

#[ensures(result)]
fn dynamic_arg(f: &dyn std::fmt::Debug) -> bool { let _ = f; true }

#[ensures(result)]
fn handle_arg(f: std::fs::File) -> bool { let _ = f; true }

#[requires(n < 0)]
#[ensures(result == n)]
#[allow(vacuous, reason = "empty domain for this probe")]
fn allowed(n: u16) -> u16 { n }

#[requires(n < 0)]
#[ensures(result == n)]
fn forbidden(n: u16) -> u16 { n }

struct Dial { phase: u8 }
impl Dial {
    pub fn new() -> Self { Self { phase: 0 } }
    #[requires(amount <= 2 && self.phase <= 8)]
    pub fn turn(&mut self, amount: u8) { self.phase += amount; }
    #[ensures(result == 0)]
    pub fn phase(&self) -> u8 { self.phase }
}

struct Sealed { value: u8 }
impl Sealed {
    #[requires(n < 0)]
    pub fn new(n: u8) -> Self { Self { value: n } }
}
#[allow(vacuous, reason = "constructor rejects every input")]
#[ensures(result)]
fn sealed(s: Sealed) -> bool { let _ = s.value; true }

#[ensures(result)]
fn ordinary_panic(n: bool) -> bool { assert!(!n, "body fault"); true }

#[ensures(result)]
fn capabilities(fs: &std::fs::Fs, clock: &std::time::Clock) -> bool {
    assert!(fs.read("absent").is_err());
    let _ = clock.now();
    true
}

mod nested {
    #[ensures(result == n)]
    pub fn unchanged(n: u32) -> u32 { n }
}

#[test]
fn plain() { assert_eq!(nested::unchanged(7), 7); }

#[requires(n <= 7)]
#[ensures(result == n)]
const fn constant(n: u16) -> u16 { n }

#[ensures(result)]
fn paired_bounds<A: Copy, B: AsRef<str>>(a: A, b: B) -> bool { let _ = (a, b); true }

#[ensures(result)]
fn repeated_caps(a: &mut std::fs::Fs, b: &mut std::fs::Fs) -> bool {
    assert!(a.read("absent").is_err());
    assert!(b.read("absent").is_err());
    true
}

#[derive(PartialEq, PartialOrd)]
pub struct PublicValue { pub number: u8 }
#[ensures(result)]
fn public_nested(values: Vec<Option<PublicValue>>) -> bool { let _ = values; true }

struct Broken { _private: () }
impl Broken {
    #[ensures(false)]
    pub fn new() -> Self { Self { _private: () } }
}
#[ensures(result)]
fn from_broken(value: Broken) -> bool { let _ = value; true }

#[requires(n > 0)]
fn prerequisite(n: u8) { let _ = n; }
struct Deeper { _private: () }
impl Deeper {
    pub fn new() -> Self { prerequisite(0); Self { _private: () } }
}
#[ensures(result)]
fn deeper_constructor(value: Deeper) -> bool { let _ = value; true }

struct NeedsDefault<T> { value: T }
impl<T: Default> NeedsDefault<T> {
    pub fn new() -> Self { Self { value: T::default() } }
}
struct NoDefault;
#[ensures(result)]
fn unavailable_constructor(value: NeedsDefault<NoDefault>) -> bool { let _ = value.value; true }

#[requires(seed <= 3)]
#[ensures(result == seed)]
fn seed_argument(seed: u64) -> u64 { seed }

#[requires(seed <= 8)]
#[ensures(result == 0)]
fn shrinking_seed(seed: u64) -> u64 { seed }

#[requires(values.len() >= 40 && values.is_sorted())]
#[ensures(result)]
fn guided_custom(values: Vec<PublicValue>) -> bool { let _ = values; true }

#[requires(value.is_some())]
#[ensures(result)]
fn custom_option(value: Option<PublicValue>) -> bool { let _ = value; true }

#[requires(values.len() >= 40)]
#[ensures(result)]
fn custom_slice(values: &[PublicValue]) -> bool { let _ = values; true }

struct ConstBuilt { value: u8 }
impl ConstBuilt {
    #[requires(n <= 3)]
    pub const fn new(n: u8) -> Self { Self { value: n } }
}
#[ensures(result)]
fn const_constructor(value: ConstBuilt) -> bool { let _ = value.value; true }

#[requires(n > // the comment must end before the bound
0 && n <= 7)]
#[ensures(result == n)]
fn comment_clause(n: u8) -> u8 { n }

#[requires("up
down".as_bytes()[2] == b'\n' && n <= 7)]
#[ensures(result == n)]
fn multiline_literal(n: u8) -> u8 { n }

#[requires(n <= 7 && { let _ = fs; true })]
#[ensures(result == n)]
fn capability_scope(fs: &std::fs::Fs, n: u8) -> u8 {
    assert!(fs.read("case-file").is_err());
    fs.write("case-file", b"same case").unwrap();
    assert_eq!(fs.read("case-file").unwrap(), b"same case");
    n
}

#[requires(n <= 7 && { let _ = fs; false })]
#[ensures(result)]
fn capability_vacuous(fs: &std::fs::Fs, n: u8) -> bool { let _ = (fs, n); true }

#[ensures(result == text.len())]
fn static_text(text: &'static str) -> usize { text.len() }

type StaticText = &'static str;
#[ensures(result == text.len())]
fn static_alias(text: StaticText) -> usize { text.len() }

#[ensures(result == text.len())]
fn bound_static<'a: 'static>(text: &'a str) -> usize { text.len() }

#[ensures(result == text.len())]
fn transitive_static<'a: 'b, 'b: 'static>(text: &'a str) -> usize { text.len() }

#[ensures(result == a.len() + b.len())]
fn related_borrows<'a: 'b, 'b>(a: &'a str, b: &'b str) -> usize { a.len() + b.len() }

struct Borrowed<'a> { value: &'a str }
impl std::test::Arbitrary for Borrowed<'static> {
    fn arbitrary(_: &mut std::test::Gen) -> Self { Self { value: "kept" } }
}
#[ensures(result)]
fn nested_borrow<'a>(value: &'a mut Borrowed<'a>) -> bool { let _ = value.value; true }

struct LifetimeWitness<T>(T);
trait StaticWitness {}
impl StaticWitness for LifetimeWitness<&'static str> {}
#[ensures(result)]
fn trait_lifetime<'a>(text: &'a str) -> bool where LifetimeWitness<&'a str>: StaticWitness {
    let _ = text;
    true
}

#[ensures(result == text.len())]
fn local_borrow<'a>(text: &'a str) -> usize { text.len() }

trait BorrowProjection { type Item; }
impl BorrowProjection for i64 { type Item = &'static str; }
#[ensures(result)]
fn projected_borrow<T: BorrowProjection>(value: T::Item) -> bool { let _ = value; true }

trait NumberProjection { type Item; }
impl NumberProjection for i64 { type Item = u16; }
#[ensures(result)]
fn projected_number<T: NumberProjection>(value: T::Item) -> bool { let _ = value; true }


#[ensures(result >= i64::MIN)]
fn argument_impl(input: impl std::convert::Into<i64>) -> i64 { input.into() }

mod exported {
    mod hidden { pub struct Meters { pub count: u8 } }
    pub use hidden::Meters;
}
#[ensures(result == value.count)]
fn reexported(value: exported::Meters) -> u8 { value.count }

mod thermometer { pub struct Level { pub value: u8 } }
mod smallalloc { pub struct Level { pub value: u8 } }
#[ensures(result == u16::from(a.value) + u16::from(b.value))]
fn crate_substrings(a: thermometer::Level, b: smallalloc::Level) -> u16 {
    u16::from(a.value) + u16::from(b.value)
}

fn local_container() {
    struct Local { number: u8 }
    impl Local {
        #[ensures(result <= u8::MAX)]
        fn local_method(&self) -> u8 { self.number }
    }
}

#[ensures(result)]
fn default_law_value() -> bool { true }
trait DefaultLaw {
    fn default_law(&self) -> bool { default_law_value() }
}

#[requires(n < 0)]
#[ensures(result == n)]
#[cfg_attr(test, allow(vacuous, reason = "cfg attribute reason"))]
#[allow(dead_code)]
fn cfg_allowed(n: u16) -> u16 { n }

// Marker text supplied by an ordinary call is data, never a generated predicate.
#[ensures(result == n)]
fn forged_const(n: u8) -> u8 {
    std::__contract::const_requires_marker("{ let p = 0usize as *const u8; let _ = unsafe { *p }; true }");
    n
}
#[ensures(result == n)]
fn forged_runtime(n: u8) -> u8 {
    std::__contract::requires(std::__contract::Clause {
        kind: std::__contract::Kind::Requires,
        text: "{ let p = 0usize as *const u8; let _ = unsafe { *p }; true }",
        function: "forged_runtime", file: "marker data", line: 0, column: 0,
        index: 0, conjunct: 0,
    }, || true);
    n
}

#[invariants]
#[requires(n <= 7)]
#[ensures(result == n)]
fn invariant_clause(n: u8) -> u8 { n }


pub struct Packet<const N: usize, const OFFSET: i16, const LIVE: bool, const MARK: char, const LIMIT: u128> {
    pub bytes: [u8; N],
}
#[ensures(result == 7)]
fn concrete_consts(value: Packet<7, { i16::MIN }, true, '>', { u128::MAX }>) -> usize { value.bytes.len() }
