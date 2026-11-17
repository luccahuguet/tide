// Reference §1 and §§3–5, independently of the held-out trials. Each row names a
// family, representative canonical keys, and a body that actually uses that family.
pub const ITEMS: &[(&str, &[&str], &str)] = &[
    (
        "section_1_bool",
        &["core::bool::<impl bool>::then_some"],
        "assert_eq!(true.then_some(7), Some(7));",
    ),
    (
        "section_1_u8_ascii",
        &["core::ascii::EscapeDefault"],
        r#"assert_eq!(b'\n'.escape_ascii().to_string(), "\\n");"#,
    ),
    (
        "section_1_char",
        &["core::char::methods::<impl char>::to_uppercase"],
        "assert_eq!('a'.to_uppercase().collect::<String>(), \"A\");",
    ),
    (
        "section_1_str",
        &["core::str::<impl str>::split_whitespace"],
        "assert_eq!(\"a b\".split_whitespace().count(), 2);",
    ),
    (
        "section_1_string",
        &["alloc::string::String::push_str"],
        "let mut s = String::from(\"a\"); s.push_str(\"b\"); assert_eq!(s, \"ab\");",
    ),
    (
        "section_1_slice",
        &[
            "core::slice::<impl [T]>::reverse",
            "alloc::slice::<impl [T]>::sort",
        ],
        "let mut a = [3, 1, 2]; let s = &mut a[..]; s.reverse(); s.sort(); assert_eq!(s, &[1, 2, 3]);",
    ),
    (
        "section_1_array",
        &["core::array::<impl [T; N]>::map"],
        "assert_eq!([1, 2].map(|n| n + 1), [2, 3]);",
    ),
    (
        "section_1_tuple",
        &["core::cmp::Ord::cmp"],
        "assert_eq!((1, 2).cmp(&(1, 3)), std::cmp::Ordering::Less);",
    ),
    (
        "section_1_option",
        &["core::option::Option::<T>::map"],
        "assert_eq!(Some(2).map(|n| n * 3), Some(6));",
    ),
    (
        "section_1_result",
        &["core::result::Result::<T, E>::map_err"],
        "let r: Result<u8, u8> = Err(2); assert_eq!(r.map_err(|e| e + 1), Err(3));",
    ),
    (
        "section_1_vec",
        &["alloc::vec::Vec::<T, A>::push"],
        "let mut v = vec![1]; v.push(2); assert_eq!(v.pop(), Some(2));",
    ),
    (
        "section_1_box",
        &["alloc::boxed::Box::<T>::new"],
        "let b = Box::new(3); assert_eq!(*b, 3);",
    ),
    (
        "section_1_rc_weak",
        &[
            "alloc::rc::Rc::<T>::new",
            "alloc::rc::Weak::<T, A>::upgrade",
        ],
        r#"
        let rc = std::rc::Rc::new(3);
        let weak = std::rc::Rc::downgrade(&rc);
        assert_eq!(*weak.upgrade().unwrap(), 3);
    "#,
    ),
    (
        "section_1_cell",
        &["core::cell::Cell::<T>::replace"],
        "let c = std::cell::Cell::new(1); assert_eq!(c.replace(2), 1); assert_eq!(c.get(), 2);",
    ),
    (
        "section_1_refcell",
        &["core::cell::RefCell::<T>::borrow_mut"],
        "let c = std::cell::RefCell::new(1); *c.borrow_mut() += 1; assert_eq!(*c.borrow(), 2);",
    ),
    (
        "section_1_oncecell",
        &["core::cell::once::OnceCell::<T>::get_or_init"],
        "let c = std::cell::OnceCell::new(); assert_eq!(*c.get_or_init(|| 3), 3);",
    ),
    (
        "section_1_vecdeque",
        &["alloc::collections::vec_deque::VecDeque::<T, A>::push_front"],
        "let mut q = std::collections::VecDeque::new(); q.push_front(1); assert_eq!(q.pop_back(), Some(1));",
    ),
    (
        "section_1_btreemap",
        &["alloc::collections::btree::map::BTreeMap::<K, V, A>::insert"],
        "let mut m = std::collections::BTreeMap::new(); m.insert(1, 2); assert_eq!(m.get(&1), Some(&2));",
    ),
    (
        "section_1_btreeset",
        &["alloc::collections::btree::set::BTreeSet::<T, A>::insert"],
        "let mut s = std::collections::BTreeSet::new(); assert!(s.insert(1)); assert!(s.contains(&1));",
    ),
    (
        "section_1_binaryheap",
        &["alloc::collections::binary_heap::BinaryHeap::<T, A>::push"],
        "let mut h = std::collections::BinaryHeap::new(); h.push(2); h.push(1); assert_eq!(h.pop(), Some(2));",
    ),
    (
        "section_1_cow",
        &["alloc::borrow::Cow::<'_, B>::to_mut"],
        "let mut c = std::borrow::Cow::Borrowed(\"a\"); c.to_mut().push('b'); assert_eq!(c, \"ab\");",
    ),
    (
        "section_1_path_pathbuf",
        &[
            "std::path::Path::join",
            "std::path::PathBuf::push",
            "std::path::Path::is_empty",
            "std::path::PathBuf::into_string",
        ],
        r#"
        let p = std::path::Path::new("a");
        let mut b = p.join("b"); b.push("c");
        assert_eq!(b.file_name().unwrap(), "c");
        assert!(!p.is_empty() && std::path::Path::new("").is_empty());
        assert_eq!(b.into_string(), Ok(String::from("a/b/c")));
    "#,
    ),
    (
        "section_1_osstr_osstring",
        &[
            "std::ffi::os_str::OsStr::to_str",
            "std::ffi::os_str::OsString::push",
        ],
        r#"
        let s = std::ffi::OsStr::new("a");
        let mut owned = s.to_os_string(); owned.push("b");
        assert_eq!(owned.to_str(), Some("ab"));
    "#,
    ),
    (
        "section_1_duration",
        &["core::time::Duration::from_millis"],
        "let d = std::time::Duration::from_millis(1500); assert_eq!(d.as_secs(), 1);",
    ),
    (
        "section_1_cursor",
        &[
            "core::io::cursor::Cursor",
            "core::io::cursor::Cursor::<T>::new",
        ],
        "let c = std::io::Cursor::new([1, 2]); assert_eq!(c.position(), 0); assert_eq!(c.into_inner(), [1, 2]);",
    ),
    (
        "section_1_seekfrom",
        &[
            "core::io::seek::SeekFrom",
            "core::io::seek::SeekFrom::Start",
        ],
        "assert!(matches!(std::io::SeekFrom::Start(2), std::io::SeekFrom::Start(2)));",
    ),
    (
        "section_1_cmp",
        &["core::cmp::min"],
        "assert_eq!(std::cmp::min(2, 3), 2);",
    ),
    (
        "section_1_ops",
        &["core::ops::range::Range::<Idx>::contains"],
        "let r: std::ops::Range<usize> = 1..3; assert!(r.contains(&2));",
    ),
    (
        "section_1_mem",
        &["core::mem::replace", "core::mem::size_of"],
        "let mut n = 1; assert_eq!(std::mem::replace(&mut n, 2), 1); assert_eq!(std::mem::size_of::<u32>(), 4);",
    ),
    (
        "section_1_fmt",
        &["alloc::fmt::format", "core::fmt::Write::write_fmt"],
        r#"
        use std::fmt::Write;
        let mut s = format!("{}", 12); write!(&mut s, "!").unwrap();
        writeln!(&mut s, "done").unwrap(); assert_eq!(s, "12!done\n");
    "#,
    ),
    (
        "section_1_convert",
        &[
            "core::convert::TryFrom::try_from",
            "core::convert::From::from",
        ],
        "assert_eq!(u8::try_from(7u16).unwrap(), 7); assert_eq!(u32::from(7u8), 7);",
    ),
    (
        "section_1_iterators",
        &["core::iter::traits::iterator::Iterator::flat_map"],
        r#"
        let v: Vec<_> = (0..4).rev().skip(1).take(2).flat_map(|n| [n, n])
            .enumerate().filter_map(|(i, n)| (i % 2 == 0).then_some(n))
            .chain(std::iter::once(9)).zip([2, 1, 9]).collect();
        assert_eq!(v, [(2, 2), (1, 1), (9, 9)]);
    "#,
    ),
    (
        "section_1_any",
        &[
            "core::any::Any",
            "<(dyn core::any::Any + 'static)>::downcast_ref",
        ],
        r#"
        let n = 7u32; let a: &dyn std::any::Any = &n;
        assert_eq!(a.downcast_ref::<u32>(), Some(&7));
    "#,
    ),
    (
        "section_1_error",
        &[
            "core::error::Error",
            "<(dyn core::error::Error + 'static)>::is",
        ],
        r#"
        let e = std::fmt::Error; let e: &dyn std::error::Error = &e;
        assert!(e.is::<std::fmt::Error>());
    "#,
    ),
    (
        "section_1_borrow",
        &["core::borrow::Borrow::borrow"],
        "let s = String::from(\"a\"); let b: &str = std::borrow::Borrow::borrow(&s); assert_eq!(b, \"a\");",
    ),
    (
        "section_1_hash",
        &[
            "core::hash::Hash::hash",
            "std::hash::random::DefaultHasher::new",
        ],
        r#"
        use std::hash::{Hash, Hasher};
        let mut a = std::hash::DefaultHasher::new(); 3u8.hash(&mut a);
        let mut b = std::hash::DefaultHasher::new(); 3u8.hash(&mut b);
        assert_eq!(a.finish(), b.finish());
    "#,
    ),
    (
        "section_2_expansions",
        &[
            "alloc::intrinsics::write_box_via_move",
            "core::intrinsics::discriminant_value",
            "std::macros::panic",
        ],
        r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        enum Kind { #[default] A, B(u8) }
        let a = Kind::default(); let b = Kind::B(3);
        assert_eq!(a, a.clone()); assert_ne!(a, b); debug_assert!(a < b);
        assert!(matches!(b, Kind::B(3))); assert_eq!(vec![a; 2].len(), 2);
        let text = format!("{b:?}"); assert_eq!(text, "B(3)");
        if text.is_empty() { panic!("empty"); }
        let _todo = || -> () { todo!("unused"); };
        let _unimplemented = || -> () { unimplemented!("unused"); };
        let _unreachable = || -> () { unreachable!("unused"); };
    "#,
    ),
    (
        "section_3_lifetimes_closures",
        &["core::ops::function::Fn", "core::marker::PhantomData"],
        r#"
        fn first<'a>(a: &'a str, _b: &str) -> &'a str { a }
        fn over<'a>(prefix: &'a str) -> impl Fn(&str) -> bool + use<'a> {
            move |s| s.starts_with(prefix)
        }
        let f: Box<dyn Fn(&str) -> bool + '_> = Box::new(over("a"));
        assert!(f(first("abc", "b")));
        let _marker: std::marker::PhantomData<std::rc::Rc<()>> = std::marker::PhantomData;
    "#,
    ),
    (
        "section_4_integer_methods",
        &[
            "core::num::<impl u32>::wrapping_add",
            "core::num::<impl i32>::cast_unsigned",
        ],
        r#"
        assert_eq!(u32::MAX.wrapping_add(1), 0);
        assert_eq!(u8::MAX.checked_add(1), None);
        assert_eq!(u16::MAX.saturating_add(1), u16::MAX);
        assert_eq!(u64::MAX.overflowing_add(1), (0, true));
        assert_eq!((-1i32).cast_unsigned(), u32::MAX);
        assert_eq!(u8::MAX.cast_signed(), -1);
        assert_eq!(i8::MIN.wrapping_sub(1), i8::MAX);
        assert_eq!(i16::MAX.checked_mul(2), None);
        assert_eq!(i64::MIN.saturating_sub(1), i64::MIN);
        assert_eq!(i128::MAX.overflowing_add(1), (i128::MIN, true));
        assert_eq!(u128::MAX.count_ones(), 128);
        assert_eq!(usize::MAX.wrapping_add(1), 0);
        assert_eq!(isize::MAX.checked_add(1), None);
    "#,
    ),
    (
        "section_4_numeric_wrappers",
        &[
            "core::num::wrapping::Wrapping",
            "core::num::saturating::Saturating",
        ],
        r#"
        use std::num::{Wrapping, Saturating};
        assert_eq!((Wrapping(u8::MAX) + Wrapping(1)).0, 0);
        assert_eq!((Saturating(u8::MAX) + Saturating(1)).0, u8::MAX);
    "#,
    ),
    (
        "section_4_f32",
        &[
            "core::f32::<impl f32>::from_bits",
            "std::f32::<impl f32>::sqrt",
            "std::f32::<impl f32>::mul_add",
            "std::f32::<impl f32>::floor",
            "std::f32::<impl f32>::ceil",
            "std::f32::<impl f32>::round",
            "std::f32::<impl f32>::trunc",
            "core::f32::<impl f32>::abs",
            "core::f32::<impl f32>::is_nan",
            "core::f32::<impl f32>::is_finite",
            "core::f32::<impl f32>::EPSILON",
            "core::f32::consts::PI",
        ],
        r#"
        assert_eq!(f32::from_bits(0), 0.0); assert_eq!(9.0f32.sqrt(), 3.0);
        assert_eq!(2.0f32.mul_add(3.0, 4.0), 10.0);
        assert_eq!((-1.5f32).floor(), -2.0); assert_eq!((-1.5f32).ceil(), -1.0);
        assert_eq!((-1.5f32).round(), -2.0); assert_eq!((-1.5f32).trunc(), -1.0);
        assert_eq!((-1.5f32).abs(), 1.5); assert!(f32::NAN.is_nan());
        assert!(1.0f32.is_finite()); assert!(f32::EPSILON < 1.0);
        assert!(std::f32::consts::PI > 3.0);
    "#,
    ),
    (
        "section_4_f64",
        &[
            "core::f64::<impl f64>::from_bits",
            "std::f64::<impl f64>::sqrt",
            "std::f64::<impl f64>::mul_add",
            "std::f64::<impl f64>::floor",
            "std::f64::<impl f64>::ceil",
            "std::f64::<impl f64>::round",
            "std::f64::<impl f64>::trunc",
            "core::f64::<impl f64>::abs",
            "core::f64::<impl f64>::is_nan",
            "core::f64::<impl f64>::is_finite",
            "core::f64::<impl f64>::MAX",
            "core::f64::consts::PI",
        ],
        r#"
        assert_eq!(f64::from_bits(0), 0.0); assert_eq!(9.0f64.sqrt(), 3.0);
        assert_eq!(2.0f64.mul_add(3.0, 4.0), 10.0);
        assert_eq!((-1.5f64).floor(), -2.0); assert_eq!((-1.5f64).ceil(), -1.0);
        assert_eq!((-1.5f64).round(), -2.0); assert_eq!((-1.5f64).trunc(), -1.0);
        assert_eq!((-1.5f64).abs(), 1.5); assert!(f64::NAN.is_nan());
        assert!(1.0f64.is_finite()); assert!(f64::MAX > 1.0);
        assert!(std::f64::consts::PI > 3.0);
    "#,
    ),
    (
        "section_5_traits",
        &[
            "core::clone::Clone::clone",
            "core::default::Default::default",
        ],
        r#"
        fn copied<T: Clone + Default + Send + Sync + Unpin>() -> T { T::default().clone() }
        trait Value { fn value(&self) -> u8; }
        struct Number(u8);
        impl Value for Number { fn value(&self) -> u8 { self.0 } }
        fn show(v: &dyn Value) -> u8 { v.value() }
        assert_eq!(copied::<u8>(), 0); assert_eq!(show(&Number(7)), 7);
        let mut n = 0; let mut f = || { n += 1; n }; assert_eq!(f(), 1);
        let s = String::from("once"); let once = move || s; assert_eq!(once(), "once");
    "#,
    ),
];

pub fn source() -> String {
    let mut source = String::new();
    for (name, _, body) in ITEMS {
        source.push_str(&format!("fn {name}() {{\n{body}\n}}\n"));
    }
    source.push_str("pub fn run() {\n");
    for (name, _, _) in ITEMS {
        source.push_str(&format!("{name}();\n"));
    }
    source.push_str("}\n");
    source
}
