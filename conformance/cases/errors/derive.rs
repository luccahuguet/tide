//@ rules: ADMIT-1
// Independently authored derive surface, shared with the executed production-std test.
#![deny(warnings)]

#[derive(Debug, Error)]
#[error("leaf {0:02x}")]
struct Leaf(u8);

#[derive(Debug, Error)]
#[error("unit {{ok}}")]
struct Unit;

#[derive(Debug, Error)]
#[error("{label}: {code:04x} {values:?}")]
struct Named<T> {
    label: &'static str,
    code: u16,
    values: T,
}

#[derive(Debug, Error)]
#[error("pair {0} {1:.2}")]
struct Pair<'a>(&'a str, f64);

#[derive(Debug, Error)]
#[error("generic {0}")]
struct Generic<T>(T);

#[derive(Debug, Error)]
#[error("array {0:?}")]
struct Array<const N: usize>([u8; N]);

#[derive(Debug, Error)]
#[error("context {tag}")]
struct Context {
    tag: &'static str,
    #[source]
    cause: Leaf,
}

#[derive(Debug, Error)]
#[error("named source")]
struct NamedSource {
    source: Leaf,
}

#[derive(Debug, Error)]
#[error("optional")]
struct Optional {
    source: Option<Leaf>,
}

#[derive(Debug, Error)]
#[error("boxed")]
struct Boxed {
    source: Box<dyn Error + Send + Sync>,
}

#[derive(Debug, Error)]
#[error(transparent)]
struct Transparent<T>(T);

#[derive(Debug, Error)]
#[error("default {}", .0)]
enum DefaultMessage {
    One(u8),
    #[error("override")]
    Two,
}

#[derive(Debug, Error)]
enum Choice<T>
where
    T: std::fmt::Debug,
{
    #[error("empty")]
    Empty,
    #[error("tuple {0:#x}")]
    Tuple(u8),
    #[error("named {value:?}")]
    Named { value: T },
    #[error("from")]
    Converted(#[from] Leaf),
    #[error("source")]
    Source { source: Leaf },
    #[error(transparent)]
    Forward(Context),
}

#[derive(Debug, Error)]
#[error("struct from")]
struct FromStruct {
    #[from]
    source: Leaf,
}

#[derive(Debug, Error)]
#[error(transparent)]
struct FromTransparent(#[from] Context);

#[derive(Debug, Error)]
#[error("expression {} {sum:03}", .0.len(), sum = .1 + 2)]
struct Expression(String, usize);

#[derive(Debug, Error)]
#[error("generic source")]
struct GenericSource<T>(#[source] T);

#[derive(Debug, Error)]
#[error("path {0}")]
struct PathError(std::path::PathBuf);

#[derive(Debug, Error)]
#[error("nested {}", .0.0)]
struct Nested((u8, u8));

#[derive(Debug, Error)]
#[error("{00}")]
struct LeadingZero(u8);

#[derive(Debug, Error)]
#[error("{} {} {} {}", if .0.is_empty() { "empty" } else { "full" }, match .1 { 0 => "zero", _ => "nonzero" }, (|| { return .1; })(), Vec::<u8>::new().len())]
struct Keywords(String, u8);

#[derive(Debug, Error)]
#[error("{} {}", .is_none, None::<fn() -> Vec<u8>>.is_none())]
struct Turbofish {
    is_none: bool,
}

#[derive(Debug, Error)]
#[error("{0} {max:02}", max = 4)]
struct Explicit(u8);

#[derive(Debug, Error)]
#[error("{__tide_error_formatter} {type:?}")]
struct Names {
    __tide_error_formatter: u8,
    r#type: u8,
}

#[derive(Debug, Error)]
enum Never {}

#[derive(Debug, Error)]
struct Manual;
impl std::fmt::Display for Manual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("manual")
    }
}

#[derive(Debug, Error)]
#[error("optional from")]
struct OptionalFrom(#[from] Option<Leaf>);

fn assert_error_type<T: Error>() {}

fn converted() -> Result<(), Choice<u8>> {
    Err(Leaf(7))?
}
fn struct_converted() -> Result<(), FromStruct> {
    Err(Leaf(9))?
}
fn assert_std_error<T: Error + Send + Sync + Pure>(_: &T) {}

fn exercise() {
    assert_error_type::<Never>();
    assert_eq!(Manual.to_string(), "manual");
    assert!(Manual.source().is_none());
    assert_eq!(
        GenericSource(Leaf(1)).source().unwrap().to_string(),
        "leaf 01"
    );
    assert_eq!(
        PathError(std::path::PathBuf::from("alpha/beta")).to_string(),
        "path alpha/beta"
    );
    assert_eq!(Nested((3, 4)).to_string(), "nested 3");
    assert_eq!(LeadingZero(7).to_string(), "7");
    assert_eq!(Keywords("x".into(), 2).to_string(), "full nonzero 2 0");
    assert_eq!(Turbofish { is_none: false }.to_string(), "false true");
    assert_eq!(Explicit(7).to_string(), "7 04");
    assert_eq!(
        Names {
            __tide_error_formatter: 3,
            r#type: 7
        }
        .to_string(),
        "3 7"
    );
    assert_eq!(
        OptionalFrom::from(Leaf(2)).source().unwrap().to_string(),
        "leaf 02"
    );
    assert_eq!(Unit.to_string(), "unit {ok}");
    assert_eq!(Leaf(10).to_string(), "leaf 0a");
    assert_eq!(
        Named {
            label: "tag",
            code: 10,
            values: vec![1, 2]
        }
        .to_string(),
        "tag: 000a [1, 2]"
    );
    assert_eq!(Pair("pi", 3.125).to_string(), "pair pi 3.12");
    assert_eq!(Generic(42).to_string(), "generic 42");
    assert_eq!(Array([1, 2]).to_string(), "array [1, 2]");
    assert_eq!(Expression("abc".into(), 5).to_string(), "expression 3 007");
    assert_eq!(DefaultMessage::One(8).to_string(), "default 8");
    assert_eq!(DefaultMessage::Two.to_string(), "override");
    assert!(Unit.source().is_none());
    let marked = Context {
        tag: "top",
        cause: Leaf(3),
    };
    assert_eq!(marked.to_string(), "context top");
    assert_eq!(
        marked.source().unwrap().downcast_ref::<Leaf>().unwrap().0,
        3
    );
    assert_eq!(
        NamedSource { source: Leaf(4) }
            .source()
            .unwrap()
            .to_string(),
        "leaf 04"
    );
    assert!(Optional { source: None }.source().is_none());
    assert_eq!(
        Optional {
            source: Some(Leaf(5))
        }
        .source()
        .unwrap()
        .to_string(),
        "leaf 05"
    );
    assert_eq!(
        Boxed {
            source: Box::new(Leaf(6))
        }
        .source()
        .unwrap()
        .downcast_ref::<Leaf>()
        .unwrap()
        .0,
        6
    );
    let transparent = Transparent(marked);
    assert_eq!(transparent.to_string(), "context top");
    assert_eq!(
        transparent
            .source()
            .unwrap()
            .downcast_ref::<Leaf>()
            .unwrap()
            .0,
        3
    );
    assert!(Transparent(Leaf(1)).source().is_none());
    assert_eq!(Choice::<u8>::Empty.to_string(), "empty");
    assert_eq!(Choice::<u8>::Tuple(31).to_string(), "tuple 0x1f");
    assert_eq!(Choice::Named { value: vec![4] }.to_string(), "named [4]");
    let e = converted().unwrap_err();
    assert_eq!(e.to_string(), "from");
    assert_eq!(e.source().unwrap().downcast_ref::<Leaf>().unwrap().0, 7);
    let direct: Choice<u8> = Leaf(8).into();
    assert_eq!(direct.source().unwrap().to_string(), "leaf 08");
    assert_eq!(
        struct_converted()
            .unwrap_err()
            .source()
            .unwrap()
            .to_string(),
        "leaf 09"
    );
    assert_eq!(
        Choice::<u8>::Source { source: Leaf(10) }
            .source()
            .unwrap()
            .to_string(),
        "leaf 0a"
    );
    let forward = Choice::<u8>::Forward(Context {
        tag: "enum",
        cause: Leaf(11),
    });
    assert_eq!(forward.to_string(), "context enum");
    assert_eq!(forward.source().unwrap().to_string(), "leaf 0b");
    let from = FromTransparent::from(Context {
        tag: "from",
        cause: Leaf(12),
    });
    assert_eq!(from.to_string(), "context from");
    assert_eq!(from.source().unwrap().to_string(), "leaf 0c");
    let io = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert_std_error(&io);
    assert_eq!(io.to_string(), "entity not found");
    assert_std_error(&std::task::TaskError::Cancelled);
    assert_eq!(
        std::task::TaskError::Cancelled.to_string(),
        "task cancelled"
    );
}

#[tide::main]
fn main() {
    exercise();
    println!("ERROR_DERIVE_RAN");
}

#[test]
fn error_derive() {
    exercise();
}
