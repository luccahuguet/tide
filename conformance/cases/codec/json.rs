//@ rules: ADMIT-1
// The Encode and Decode derives and std::json, shared with the executed production-std test.
#![deny(warnings)]

use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::float;
use std::fmt::Debug;
use std::json;
use std::rand::Rand;

fn text<T: Encode + ?Sized>(value: &T) -> String {
    String::from_utf8(json::encode(value)).unwrap()
}

/// `value` encodes to exactly `expected`, which decodes back to it.
fn round<T: Encode + Decode + PartialEq + Debug>(value: T, expected: &str) {
    assert_eq!(text(&value), expected);
    assert_eq!(json::decode::<T>(expected.as_bytes()).unwrap(), value);
}

fn fails<T: Decode + Debug>(input: &[u8]) -> String {
    json::decode::<T>(input).unwrap_err().to_string()
}

fn fails_with<T: Decode + Debug>(input: &[u8], part: &str) {
    let message = fails::<T>(input);
    assert!(message.contains(part), "{message}");
}

// ---- shapes

#[derive(Debug, PartialEq, Encode, Decode)]
struct Named {
    a: i32,
    b: i32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Pair(i32, i32);

#[derive(Debug, PartialEq, Encode, Decode)]
struct Wrap(i32);

#[derive(Debug, PartialEq, Encode, Decode)]
struct Unit;

#[derive(Debug, PartialEq, Encode, Decode)]
struct Nothing();

#[derive(Debug, PartialEq, Encode, Decode)]
struct Blank {}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Shape {
    W { a: i32, b: i32 },
    X(i32, i32),
    Y(i32),
    Z,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Command {
    Set { key: String },
    Clear,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Never {}

fn shapes() {
    round(Named { a: 1, b: -2 }, r#"{"a":1,"b":-2}"#);
    round(Pair(1, 2), "[1,2]");
    round(Wrap(7), "7");
    round(Unit, "null");
    round(Nothing(), "[]");
    round(Blank {}, "{}");
    round(Shape::W { a: 0, b: 0 }, r#"{"W":{"a":0,"b":0}}"#);
    round(Shape::X(0, 0), r#"{"X":[0,0]}"#);
    round(Shape::Y(0), r#"{"Y":0}"#);
    round(Shape::Z, r#""Z""#);
    round(Command::Set { key: "a".into() }, r#"{"Set":{"key":"a"}}"#);
    round(Command::Clear, r#""Clear""#);
    round(vec![Command::Clear, Command::Clear], r#"["Clear","Clear"]"#);

    // Whitespace is allowed around every token; fields come in any order.
    assert_eq!(
        json::decode::<Named>(b" {\t\"b\" : 2 ,\r\n \"a\" : 1 } \n").unwrap(),
        Named { a: 1, b: 2 }
    );
    fails_with::<Pair>(b"[1]", "invalid length");
    fails_with::<Pair>(b"[1,2,3]", "invalid length");
    fails_with::<Named>(b"[1,2]", "invalid type: sequence, expected a map");
    fails_with::<Unit>(b"0", "invalid type: number, expected null");

    fails_with::<Shape>(
        br#""V""#,
        "unknown variant `V`, expected one of `W`, `X`, `Y`, `Z`",
    );
    fails_with::<Shape>(br#""W""#, "variant `W`");
    fails_with::<Shape>(br#"{"Z":null}"#, "unit variant");
    fails_with::<Shape>(b"{}", "expected one of `W`, `X`, `Y`, `Z`");
    fails_with::<Shape>(
        b"7",
        "invalid type: number, expected one of `W`, `X`, `Y`, `Z`",
    );
    fails_with::<Shape>(br#"{"Y":0,"X":[0,0]}"#, "single-key");
    fails_with::<Shape>(br#"{"Y":0"#, "EOF while parsing an object");
    fails_with::<Never>(br#""A""#, "the enum has none");
}

// ---- missing, null, duplicate, and unknown keys

type Maybe<T> = Option<T>;

#[derive(Debug, PartialEq, Encode, Decode)]
struct Profile {
    name: String,
    age: Option<u8>,
    nick: Maybe<String>,
    boxed: Box<Option<u8>>,
    #[codec(default)]
    level: u8,
}

fn keys() {
    let plain = Profile {
        name: "n".into(),
        age: None,
        nick: None,
        boxed: Box::new(None),
        level: 0,
    };
    assert_eq!(json::decode::<Profile>(br#"{"name":"n"}"#).unwrap(), plain);
    round(
        plain,
        r#"{"name":"n","age":null,"nick":null,"boxed":null,"level":0}"#,
    );
    fails_with::<Profile>(b"{}", "missing field `name`");
    fails_with::<Profile>(br#"{"name":null}"#, "invalid type: null, expected a string");
    // `default` covers an absent key, not `null`.
    fails_with::<Profile>(
        br#"{"name":"n","level":null}"#,
        "invalid type: null, expected u8",
    );
    fails_with::<Profile>(br#"{"name":"a","name":"b"}"#, "duplicate field `name`");
    round(Some(Some(3u8)), "3");
    assert_eq!(text(&Some(None::<u8>)), "null");
    assert_eq!(json::decode::<Option<Option<u8>>>(b"null").unwrap(), None);

    let unknown = br#"{"extra":{"deep":[1,{"x":null}],"s":"\u00e9"},"name":"n","more":[true,false,1.5e3],"more":0}"#;
    assert_eq!(json::decode::<Profile>(unknown).unwrap().name, "n");
    // An ignored value is still checked, and its nesting counts.
    fails_with::<Profile>(br#"{"x":tru,"name":"n"}"#, "expected ident");
    fails_with::<Profile>(br#"{"x":"\ud800","name":"n"}"#, "surrogate");
    let nested = |depth: usize| {
        let mut input = String::from(r#"{"x":"#);
        input.push_str(&"[".repeat(depth));
        input.push_str(&"]".repeat(depth));
        input.push_str(r#","name":"n"}"#);
        json::decode::<Profile>(input.as_bytes())
    };
    assert!(nested(127).is_ok());
    assert!(nested(128).is_err());
    let deep = nested(200).unwrap_err().to_string();
    assert!(deep.starts_with("recursion limit exceeded"), "{deep}");
}

// ---- attributes

#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
struct Renamed {
    first_name: String,
    #[codec(rename = "ID")]
    user_id: u32,
    r#type: u8,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "lowercase")]
struct FieldLower {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "UPPERCASE")]
struct FieldUpper {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "PascalCase")]
struct FieldPascal {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
struct FieldCamel {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "snake_case")]
struct FieldSnake {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "SCREAMING_SNAKE_CASE")]
struct FieldScreaming {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "kebab-case")]
struct FieldKebab {
    very_tasty: u8,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "SCREAMING-KEBAB-CASE")]
struct FieldScreamingKebab {
    very_tasty: u8,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "lowercase")]
enum VariantLower {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "UPPERCASE")]
enum VariantUpper {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "PascalCase")]
enum VariantPascal {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
enum VariantCamel {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "snake_case")]
enum VariantSnake {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "SCREAMING_SNAKE_CASE")]
enum VariantScreaming {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "kebab-case")]
enum VariantKebab {
    VeryTasty,
}
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "SCREAMING-KEBAB-CASE")]
enum VariantScreamingKebab {
    VeryTasty,
}

// `rename_all` on an enum renames its variants, not their fields.
#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(rename_all = "snake_case")]
enum Event {
    #[codec(rename = "START")]
    Started,
    UserJoined {
        #[codec(rename = "userName")]
        user_name: String,
        #[codec(default)]
        is_guest: bool,
    },
    Batch(Vec<u8>, bool),
    Empty {},
    Hollow(),
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(default)]
struct Settings {
    width: u32,
    title: String,
    #[codec(skip)]
    cache: Vec<u8>,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            width: 80,
            title: "untitled".into(),
            cache: vec![1],
        }
    }
}

#[derive(Debug, PartialEq, Decode)]
#[codec(default)]
struct Defaults<T> {
    inherited: u8,
    #[codec(default)]
    explicit: T,
    #[codec(skip, default)]
    skipped: T,
    #[codec(skip)]
    cache: u8,
}

impl<T: From<u8>> Default for Defaults<T> {
    fn default() -> Self {
        Self {
            inherited: 7,
            explicit: T::from(9),
            skipped: T::from(11),
            cache: 13,
        }
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[codec(default)]
struct DefaultPair<T>(u8, T);

impl<T: From<u8>> Default for DefaultPair<T> {
    fn default() -> Self {
        Self(7, T::from(9))
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Session {
    user: String,
    #[codec(skip)]
    token: Option<String>,
    #[codec(skip, default)]
    hits: u32,
}

#[derive(Debug, PartialEq)]
struct NoDefault(u8);

// With `Encode` alone, a skipped field needs no `Default`, and a borrowed field encodes.
#[derive(Encode)]
struct Report<'a> {
    title: &'a str,
    #[codec(skip)]
    scratch: NoDefault,
}

fn attributes() {
    round(
        Renamed {
            first_name: "a".into(),
            user_id: 7,
            r#type: 1,
        },
        r#"{"firstName":"a","ID":7,"type":1}"#,
    );
    round(FieldLower { very_tasty: 1 }, r#"{"very_tasty":1}"#);
    round(FieldUpper { very_tasty: 1 }, r#"{"VERY_TASTY":1}"#);
    round(FieldPascal { very_tasty: 1 }, r#"{"VeryTasty":1}"#);
    round(FieldCamel { very_tasty: 1 }, r#"{"veryTasty":1}"#);
    round(FieldSnake { very_tasty: 1 }, r#"{"very_tasty":1}"#);
    round(FieldScreaming { very_tasty: 1 }, r#"{"VERY_TASTY":1}"#);
    round(FieldKebab { very_tasty: 1 }, r#"{"very-tasty":1}"#);
    round(FieldScreamingKebab { very_tasty: 1 }, r#"{"VERY-TASTY":1}"#);
    round(VariantLower::VeryTasty, r#""verytasty""#);
    round(VariantUpper::VeryTasty, r#""VERYTASTY""#);
    round(VariantPascal::VeryTasty, r#""VeryTasty""#);
    round(VariantCamel::VeryTasty, r#""veryTasty""#);
    round(VariantSnake::VeryTasty, r#""very_tasty""#);
    round(VariantScreaming::VeryTasty, r#""VERY_TASTY""#);
    round(VariantKebab::VeryTasty, r#""very-tasty""#);
    round(VariantScreamingKebab::VeryTasty, r#""VERY-TASTY""#);

    round(Event::Started, r#""START""#);
    round(
        Event::UserJoined {
            user_name: "x".into(),
            is_guest: true,
        },
        r#"{"user_joined":{"userName":"x","is_guest":true}}"#,
    );
    assert_eq!(
        json::decode::<Event>(br#"{"user_joined":{"userName":"x"}}"#).unwrap(),
        Event::UserJoined {
            user_name: "x".into(),
            is_guest: false
        }
    );
    round(
        Event::Batch(vec![1, 2], false),
        r#"{"batch":[[1,2],false]}"#,
    );
    round(Event::Empty {}, r#"{"empty":{}}"#);
    round(Event::Hollow(), r#"{"hollow":[]}"#);
    fails_with::<Event>(br#""Started""#, "unknown variant `Started`");
    fails_with::<Event>(
        br#"{"user_joined":{"user_name":"x"}}"#,
        "missing field `userName`",
    );

    // A struct's `default` fills each missing field from `Self::default()`.
    assert_eq!(
        json::decode::<Settings>(br#"{"title":"t"}"#).unwrap(),
        Settings {
            width: 80,
            title: "t".into(),
            cache: vec![1]
        }
    );
    assert_eq!(
        text(&Settings::default()),
        r#"{"width":80,"title":"untitled"}"#
    );
    for (input, explicit) in [("{}", 0), (r#"{"explicit":5,"skipped":99}"#, 5)] {
        assert_eq!(
            json::decode::<Defaults<u8>>(input.as_bytes()).unwrap(),
            Defaults {
                inherited: 7,
                explicit,
                skipped: 0,
                cache: 13,
            }
        );
    }
    for (input, expected) in [
        ("[]", DefaultPair(7, 9u8)),
        ("[1]", DefaultPair(1, 9)),
        ("[1,2]", DefaultPair(1, 2)),
    ] {
        assert_eq!(
            json::decode::<DefaultPair<u8>>(input.as_bytes()).unwrap(),
            expected
        );
    }
    round(DefaultPair(1, 2u8), "[1,2]");
    // A short tuple closes its own array, including when nested in another container.
    assert_eq!(
        json::decode::<Vec<DefaultPair<u8>>>(b"[[],[1],[2,3]]").unwrap(),
        [DefaultPair(7, 9), DefaultPair(1, 9), DefaultPair(2, 3)]
    );
    fails_with::<DefaultPair<u8>>(b"[1,2,3]", "invalid length");
    for input in ["[", "[1", "[1,]", "[1 2]", "[,1]", "[1,2,]", "null"] {
        assert!(json::decode::<DefaultPair<u8>>(input.as_bytes()).is_err());
    }

    // A skipped field is not written, takes its default, and its key is an unknown one.
    let session = Session {
        user: "u".into(),
        token: Some("secret".into()),
        hits: 3,
    };
    assert_eq!(text(&session), r#"{"user":"u"}"#);
    assert_eq!(
        json::decode::<Session>(br#"{"user":"u","token":"x","hits":"many"}"#).unwrap(),
        Session {
            user: "u".into(),
            token: None,
            hits: 0
        }
    );
    let report = Report {
        title: "t",
        scratch: NoDefault(1),
    };
    assert_eq!(report.scratch, NoDefault(1));
    assert_eq!(text(&report), r#"{"title":"t"}"#);
}

// ---- malformed input

fn malformed() {
    assert_eq!(
        fails::<i32>(b""),
        "EOF while parsing a value at line 1 column 1"
    );
    assert_eq!(
        fails::<Vec<i32>>(b"[1,]"),
        "trailing comma at line 1 column 4"
    );
    assert_eq!(
        fails::<i32>(b"1 2"),
        "trailing characters at line 1 column 3"
    );
    assert_eq!(
        fails::<Named>(b"{\n  \"a\": 1,\n  \"b\": x\n}"),
        "expected value at line 3 column 8"
    );
    let error = json::decode::<Named>(b"{\n  \"a\": 1,\n  \"b\": x\n}").unwrap_err();
    assert_eq!((error.line(), error.column()), (3, 8));

    fails_with::<Named>(b"{", "EOF while parsing an object");
    fails_with::<Named>(br#"{"a":1,}"#, "trailing comma");
    fails_with::<Named>(br#"{"a" 1}"#, "expected `:`");
    fails_with::<Named>(b"{a:1}", "key must be a string");
    fails_with::<Named>(br#"{"a":1 "b":2}"#, "expected `,` or `}`");
    fails_with::<Named>(br#"{"a":1,"b":2}x"#, "trailing characters");
    fails_with::<Vec<i32>>(b"[", "EOF while parsing a list");
    fails_with::<Vec<i32>>(b"[1 2]", "expected `,` or `]`");
    fails_with::<Vec<i32>>(b"[,1]", "expected value");
    fails_with::<String>(br#""\ud800""#, "lone leading surrogate");
    fails_with::<String>(br#""\ud800A""#, "lone leading surrogate");
    fails_with::<String>(br#""\udc00""#, "lone trailing surrogate");
    fails_with::<String>(b"\"a\nb\"", "control character");
    fails_with::<String>(b"\"\xff\"", "invalid unicode code point");
    fails_with::<String>(b"\"\xc3\"", "invalid unicode code point");
    fails_with::<String>(br#""abc"#, "EOF while parsing a string");
    fails_with::<String>(br#""\x""#, "invalid escape");
    fails_with::<String>(br#""\u12"#, "EOF while parsing a string");
    fails_with::<String>(br#""\u12g4""#, "invalid escape");
    fails_with::<f64>(b"01", "invalid number");
    fails_with::<f64>(b"1.", "EOF while parsing a value");
    fails_with::<f64>(b"1.e3", "invalid number");
    fails_with::<f64>(b"1e", "EOF while parsing a value");
    fails_with::<f64>(b"-", "EOF while parsing a value");
    fails_with::<f64>(b"-x", "invalid number");
    fails_with::<f64>(b".5", "expected value");
    fails_with::<f64>(b"+1", "expected value");
    fails_with::<f64>(b"NaN", "expected value");
    fails_with::<f64>(b"Infinity", "expected value");
    fails_with::<f64>(b"0x10", "trailing characters");
    fails_with::<i32>(b"\xef\xbb\xbf1", "expected value");
    fails_with::<bool>(b"tru", "EOF while parsing a value");
    fails_with::<bool>(b"trux", "expected ident");
    fails_with::<Option<i32>>(b"nul", "EOF while parsing a value");
    fails_with::<bool>(b"1", "invalid type: number, expected a boolean");
    fails_with::<char>(br#""xy""#, "one character");

    // Typed values nest 128 deep: each tree level is an object and an array.
    let tree = |levels: usize| {
        let mut input = r#"{"value":0,"kids":["#.repeat(levels);
        input.push_str(&"]}".repeat(levels));
        json::decode::<Tree<u8>>(input.as_bytes())
    };
    assert!(tree(64).is_ok());
    let deep = tree(65).unwrap_err().to_string();
    assert!(deep.starts_with("recursion limit exceeded"), "{deep}");
}

// ---- numbers

fn float_round(x: f64) {
    let back: f64 = json::decode(&json::encode(&x)).unwrap();
    assert_eq!(float::bits(back), float::bits(x), "{x:?}");
}

fn float_round_32(x: f32) {
    let back: f32 = json::decode(&json::encode(&x)).unwrap();
    assert_eq!(float::bits(back), float::bits(x), "{x:?}");
}

fn numbers() {
    for x in [
        0.0,
        -0.0,
        0.1,
        1.0,
        5e-324,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        1e16,
        1e-7,
        1e300,
        123456789.125,
        9007199254740993.0,
    ] {
        float_round(x);
    }
    for x in [0.1f32, -0.0, 1e-45, f32::MAX, f32::MIN_POSITIVE, 16777216.0] {
        float_round_32(x);
    }
    // A fixed xorshift sequence over all bit patterns.
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut checked = 0;
    while checked < 4000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let x = f64::from_bits(state);
        let y = f32::from_bits((state >> 32) as u32);
        if x.is_finite() && y.is_finite() {
            float_round(x);
            float_round_32(y);
            checked += 1;
        }
    }
    assert_eq!(text(&1.0f64), "1.0");
    assert_eq!(text(&-0.0f64), "-0.0");
    assert_eq!(text(&1e16f64), "1e16");
    assert_eq!(text(&1e-7f64), "1e-7");
    assert_eq!(text(&0.1f32), "0.1");
    assert_eq!(
        text(&vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY]),
        "[null,null,null]"
    );
    assert_eq!(text(&f32::NAN), "null");
    assert_eq!(json::decode::<Option<f64>>(b"null").unwrap(), None);
    fails_with::<f64>(b"null", "invalid type: null, expected f64");
    assert_eq!(
        fails::<f64>(b"1e400"),
        "number out of range at line 1 column 1"
    );
    fails_with::<f32>(b"1e39", "number out of range");
    assert_eq!(json::decode::<f64>(b"1").unwrap(), 1.0);
    assert_eq!(json::decode::<f64>(b"-1.5E+2").unwrap(), -150.0);
    assert_eq!(
        float::bits(json::decode::<f64>(b"-0").unwrap()),
        float::bits(-0.0f64)
    );
    // An f32 is rounded once, from the decimal text.
    assert_eq!(json::decode::<f32>(b"16777217").unwrap(), 16777216.0);

    round(u64::MAX, "18446744073709551615");
    round(i64::MIN, "-9223372036854775808");
    round(u128::MAX, "340282366920938463463374607431768211455");
    round(i128::MIN, "-170141183460469231731687303715884105728");
    round(usize::MAX, &usize::MAX.to_string());
    round(-7isize, "-7");
    round(255u8, "255");
    round(true, "true");
    fails_with::<u8>(b"256", "number out of range");
    fails_with::<u8>(b"-1", "number out of range");
    fails_with::<i8>(b"-129", "number out of range");
    assert_eq!(json::decode::<u8>(b"-0").unwrap(), 0);
    fails_with::<i32>(b"1.0", "invalid type: floating point `1.0`, expected i32");
    fails_with::<i32>(b"1e2", "floating point");
    fails_with::<i32>(br#""1""#, "invalid type: string, expected i32");
}

// ---- strings

fn strings() {
    let all: String = (0u8..0x20)
        .map(char::from)
        .chain("\"\\/\u{7f}\u{e9}\u{1f600}".chars())
        .collect();
    let encoded = json::encode(&all);
    assert_eq!(json::decode::<String>(&encoded).unwrap(), all);
    // Only the quote, the backslash, and the control characters are escaped.
    assert_eq!(
        text("a\"b\\c/\n\u{1}\u{1f}\u{7f}\u{e9}\u{1f600}"),
        "\"a\\\"b\\\\c/\\n\\u0001\\u001f\u{7f}\u{e9}\u{1f600}\""
    );
    assert_eq!(text("\u{8}\t\n\u{c}\r"), r#""\b\t\n\f\r""#);
    assert_eq!(
        json::decode::<String>(br#""\u00e9\ud83d\ude00\/\b\f\n\r\t\"\\\u0000""#).unwrap(),
        "\u{e9}\u{1f600}/\u{8}\u{c}\n\r\t\"\\\0"
    );
    round('x', r#""x""#);
    round('\u{1f600}', "\"\u{1f600}\"");
    round(String::new(), r#""""#);
}

// ---- std types

fn std_types() {
    let bytes = vec![7u8; 1000];
    assert_eq!(
        json::decode::<Vec<u8>>(&json::encode(&bytes)).unwrap(),
        bytes
    );
    let named: Vec<_> = (0..300).map(|a| Named { a, b: -a }).collect();
    assert_eq!(
        json::decode::<Vec<Named>>(&json::encode(&named)).unwrap(),
        named
    );
    round((1u8, "a".to_string(), true), r#"[1,"a",true]"#);
    round((1u8,), "[1]");
    round(
        ((), Box::new(2i32), 'c', Some(false)),
        r#"[null,2,"c",false]"#,
    );
    fails_with::<(u8, u8)>(b"[1]", "invalid length, expected an array of 2 elements");
    fails_with::<(u8, u8)>(b"[1,2,3]", "invalid length");
    round(vec![Some(1), None], "[1,null]");
    round(VecDeque::from(vec![1u8, 2]), "[1,2]");
    round(Vec::<u8>::new(), "[]");
    assert_eq!(json::decode::<Vec<u8>>(b" \n[ 1 , 2 ]\t").unwrap(), [1, 2]);
    assert_eq!(text(&[1u8, 2][..]), "[1,2]");
    assert_eq!(text("s"), r#""s""#);
    assert_eq!(text(&&7u8), "7");
}

fn entries() -> [(String, i32); 4] {
    [
        ("b".to_string(), 1),
        ("a".to_string(), 2),
        ("\u{e9}".to_string(), 3),
        ("B".to_string(), 4),
    ]
}

const SORTED: &str = "{\"B\":4,\"a\":2,\"b\":1,\"\u{e9}\":3}";

fn maps() {
    let mut forward = HashMap::new();
    forward.extend(entries());
    let mut backward = HashMap::with_capacity(1000);
    backward.extend(entries().into_iter().rev());
    // Equal maps encode to the same bytes, in key order, whatever their history.
    assert_eq!(text(&forward), SORTED);
    assert_eq!(text(&backward), SORTED);
    assert_eq!(
        json::decode::<HashMap<String, i32>>(SORTED.as_bytes()).unwrap(),
        forward
    );
    round(BTreeMap::from(entries()), SORTED);
    round(BTreeMap::<String, Vec<Option<bool>>>::new(), "{}");
    let mut nested = HashMap::new();
    nested.insert("k".to_string(), vec![Some(Named { a: 1, b: 2 }), None]);
    round(nested, r#"{"k":[{"a":1,"b":2},null]}"#);
    fails_with::<HashMap<String, i32>>(br#"{"a":1,"a":2}"#, "duplicate key `a`");
    fails_with::<BTreeMap<String, i32>>(br#"{"a":1,"a":2}"#, "duplicate key `a`");
    fails_with::<BTreeMap<String, i32>>(b"[]", "invalid type: sequence, expected a map");
}

fn seeded_map(rand: &Rand) {
    let mut seeded = HashMap::with_hasher(RandomState::from_rand(rand));
    seeded.extend(entries());
    assert_eq!(text(&seeded), SORTED);
}

// ---- generics

#[derive(Debug, PartialEq, Encode, Decode)]
struct Wrapper<T> {
    inner: T,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Tree<T> {
    value: T,
    kids: Vec<Tree<T>>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum Either<L, R>
where
    L: Clone,
{
    Left(L),
    Right { value: R },
}

// `T` is in a skipped field only: it needs neither trait, and `Option<T>` has its `Default`.
#[derive(Debug, PartialEq, Encode, Decode)]
struct Lazy<T> {
    value: u8,
    #[codec(skip)]
    memo: Option<T>,
}

#[derive(Debug, PartialEq, Default, Encode, Decode)]
#[codec(default)]
struct Limits<T> {
    low: T,
    high: T,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Grid<const N: usize> {
    cells: Vec<u8>,
}

trait Item {
    type Value;
}

struct Provider;

impl Item for Provider {
    type Value = u8;
}

#[derive(Encode, Decode)]
struct Projection<T: Item> {
    shorthand: T::Value,
    qualified: <T as Item>::Value,
}

fn pure_text<T: Pure + Encode>(value: &T) -> String {
    text(value)
}

fn pure_decode<T: Pure + Decode>(input: &str) -> T {
    json::decode(input.as_bytes()).unwrap()
}

fn generics() {
    round(Wrapper { inner: 5u8 }, r#"{"inner":5}"#);
    round(
        Wrapper {
            inner: Wrapper {
                inner: Some("x".to_string()),
            },
        },
        r#"{"inner":{"inner":"x"}}"#,
    );
    let tree = Tree {
        value: 1u8,
        kids: vec![Tree {
            value: 2,
            kids: vec![],
        }],
    };
    assert_eq!(
        pure_text(&tree),
        r#"{"value":1,"kids":[{"value":2,"kids":[]}]}"#
    );
    assert_eq!(pure_decode::<Tree<u8>>(&pure_text(&tree)), tree);
    round(Either::<u8, String>::Left(1), r#"{"Left":1}"#);
    round(
        Either::<u8, String>::Right { value: "r".into() },
        r#"{"Right":{"value":"r"}}"#,
    );
    let lazy = Lazy {
        value: 1,
        memo: Some(NoDefault(2)),
    };
    assert_eq!(text(&lazy), r#"{"value":1}"#);
    assert_eq!(
        json::decode::<Lazy<NoDefault>>(br#"{"value":1}"#).unwrap(),
        Lazy {
            value: 1,
            memo: None
        }
    );
    assert_eq!(
        json::decode::<Limits<i8>>(br#"{"high":9}"#).unwrap(),
        Limits { low: 0, high: 9 }
    );
    round(Grid::<3> { cells: vec![1] }, r#"{"cells":[1]}"#);
    // Only the associated values need codec impls; the provider has neither.
    let projected = Projection::<Provider> {
        shorthand: 3,
        qualified: 5,
    };
    let encoded = json::encode(&projected);
    assert_eq!(encoded, br#"{"shorthand":3,"qualified":5}"#);
    let decoded: Projection<Provider> = json::decode(&encoded).unwrap();
    assert_eq!(decoded.shorthand, projected.shorthand);
    assert_eq!(decoded.qualified, projected.qualified);
}

// ---- the error

#[derive(Debug, PartialEq, Decode)]
pub struct Entry {
    pub title: String,
    pub link: String,
}

#[derive(Debug, Error)]
pub enum FeedError {
    #[error("bad feed: {0}")]
    Parse(#[from] json::Error),
}

pub fn parse_feed(body: &[u8]) -> Result<Vec<Entry>, FeedError> {
    Ok(json::decode(body)?)
}

fn assert_std_error<T: Error + Send + Sync + Pure + 'static>() {}

fn errors() {
    assert_std_error::<json::Error>();
    let feed = parse_feed(br#"[{"title":"t","link":"l","seen":true}]"#).unwrap();
    assert_eq!(
        feed,
        [Entry {
            title: "t".into(),
            link: "l".into()
        }]
    );
    let error = parse_feed(b"[").unwrap_err();
    assert_eq!(
        error.to_string(),
        "bad feed: EOF while parsing a list at line 1 column 2"
    );
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<json::Error>()
        .unwrap();
    assert_eq!((source.line(), source.column()), (1, 2));
    assert!(source.source().is_none());
    assert!(format!("{source:?}").contains("EOF while parsing a list"));
}

fn exercise() {
    shapes();
    keys();
    attributes();
    malformed();
    numbers();
    strings();
    std_types();
    maps();
    generics();
    errors();
}

#[tide::main]
fn main(root: Root) {
    exercise();
    seeded_map(&root.rand);
    println!("CODEC_RAN");
}

#[test]
fn codec() {
    exercise();
}
