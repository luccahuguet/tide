//@ rules: ADMIT-1, PURE-1
// Also built and run twice against production std by tide-admit/tests/build.rs.
use std::collections::hash_map::{Entry, RandomState};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};

fn pure<T: Pure>(_: &T) {}

fn maps() {
    let mut map = HashMap::new();
    assert!(map.is_empty());
    assert_eq!(map.insert(String::from("oak"), 2), None);
    assert_eq!(map.insert(String::from("oak"), 3), Some(2));
    assert_eq!(map.get("missing"), None);
    assert_eq!(map.get_mut("missing"), None);
    assert!(!map.contains_key("missing"));
    assert_eq!(map.remove("missing"), None);
    *map.get_mut("oak").unwrap() += 1;
    assert_eq!(map.get_key_value("oak"), Some((&String::from("oak"), &4)));
    assert!(map.contains_key("oak"));
    assert_eq!(map.get_key_value("missing"), None);
    match map.entry(String::from("elm")) {
        Entry::Vacant(e) => {
            e.insert(5);
        }
        Entry::Occupied(_) => panic!("unexpected entry"),
    }
    match map.entry(String::from("oak")) {
        Entry::Occupied(mut e) => {
            *e.get_mut() += 1;
        }
        Entry::Vacant(_) => panic!("missing entry"),
    }
    for (_, value) in map.iter_mut() {
        *value += 1;
    }
    for (_, value) in &mut map {
        *value += 1;
    }
    for value in map.values_mut() {
        *value += 1;
    }
    assert_eq!(map["oak"], 8);
    assert_eq!(map.values().sum::<i32>(), 16);
    assert_eq!(map.keys().count(), 2);
    assert_eq!(map.iter().count(), 2);
    assert_eq!((&map).into_iter().count(), 2);
    pure(&map);
    assert_eq!(map.clone(), map);
    assert!(!format!("{map:?}").is_empty());
    assert_eq!(map.clone().into_keys().count(), 2);
    assert_eq!(map.clone().into_values().sum::<i32>(), 16);
    assert_eq!(map.clone().into_iter().count(), 2);
    map.retain(|key, value| {
        *value += 1;
        key == "oak"
    });
    assert_eq!(map.len(), 1);
    assert_eq!(map.remove("oak"), Some(9));
    assert!(map.is_empty());
    map.insert(String::from("fir"), 1);
    map.clear();
    assert!(map.is_empty());

    let mut custom: HashMap<i32, i32, BuildHasherDefault<DefaultHasher>> =
        HashMap::with_hasher(BuildHasherDefault::default());
    custom.extend([(1, 2)]);
    let source = [(3, 4)];
    custom.extend(source.iter().map(|(k, v)| (k, v)));
    assert_eq!(custom[&3], 4);
    let copied: HashMap<_, _, BuildHasherDefault<DefaultHasher>> = custom.into_iter().collect();
    assert_eq!(copied[&1], 2);
    pure(&copied);

    let mut sized = HashMap::with_capacity_and_hasher(8, RandomState::default());
    assert!(sized.capacity() >= 8);
    assert_eq!(
        sized.hasher().hash_one(1),
        RandomState::default().hash_one(1)
    );
    sized.reserve(16);
    assert!(sized.capacity() >= 16);
    sized.try_reserve(32).unwrap();
    assert!(sized.capacity() >= 32);
    assert!(sized.try_reserve(usize::MAX).is_err());
    sized.extend([(1, 10), (2, 20), (3, 30)]);
    let [first, missing, second] = sized.get_disjoint_mut([&1, &0, &2]);
    *first.unwrap() += 1;
    assert!(missing.is_none());
    *second.unwrap() += 2;
    assert_eq!(sized.get_disjoint_mut::<i32, 0>([]), []);
    assert_eq!(sized.remove_entry(&0), None);
    assert_eq!(sized.remove_entry(&1), Some((1, 11)));
    let extracted: std::collections::hash_map::ExtractIf<'_, _, _, _> =
        sized.extract_if(|key, value| {
            *value += 1;
            *key == 2
        });
    assert_eq!(extracted.collect::<Vec<_>>(), [(2, 23)]);
    assert_eq!(sized[&3], 31);
    sized.shrink_to(8);
    assert!(sized.capacity() >= 8);
    sized.shrink_to_fit();
    assert!(sized.capacity() >= sized.len());
    let mut cloned = HashMap::new();
    cloned.clone_from(&sized);
    assert_eq!(cloned, sized);
    let drained: std::collections::hash_map::Drain<'_, _, _> = sized.drain();
    assert_eq!(drained.collect::<Vec<_>>(), [(3, 31)]);
    assert!(sized.is_empty());
}

fn sets() {
    let mut set = HashSet::new();
    assert!(set.is_empty());
    assert!(set.insert(String::from("oak")));
    assert!(!set.insert(String::from("oak")));
    assert!(set.contains("oak"));
    assert!(!set.contains("elm"));
    assert!(!set.remove("elm"));
    assert_eq!(set.len(), 1);
    assert_eq!(set.iter().count(), 1);
    assert_eq!((&set).into_iter().count(), 1);
    assert_eq!(set.clone().into_iter().count(), 1);
    pure(&set);
    assert_eq!(set.clone(), set);
    assert!(!format!("{set:?}").is_empty());
    set.retain(|key| key == "elm");
    assert!(set.is_empty());
    set.insert(String::from("fir"));
    assert!(set.remove("fir"));
    set.insert(String::from("elm"));
    set.clear();
    assert!(set.is_empty());
    let mut custom = HashSet::with_hasher(BuildHasherDefault::<DefaultHasher>::default());
    custom.extend([1, 2]);
    custom.extend([2, 3].iter());
    assert_eq!(custom.len(), 3);
    let copied: HashSet<_, BuildHasherDefault<DefaultHasher>> = custom.into_iter().collect();
    assert!(copied.contains(&3));

    let mut sized = HashSet::with_capacity_and_hasher(8, RandomState::default());
    assert!(sized.capacity() >= 8);
    assert_eq!(
        sized.hasher().hash_one(1),
        RandomState::default().hash_one(1)
    );
    sized.reserve(16);
    assert!(sized.capacity() >= 16);
    sized.try_reserve(32).unwrap();
    assert!(sized.capacity() >= 32);
    assert!(sized.try_reserve(usize::MAX).is_err());
    assert_eq!(sized.replace(String::from("oak")), None);
    assert_eq!(
        sized.replace(String::from("oak")),
        Some(String::from("oak"))
    );
    assert_eq!(sized.get("oak").map(String::as_str), Some("oak"));
    assert_eq!(sized.get("missing"), None);
    assert_eq!(sized.take("missing"), None);
    assert_eq!(sized.take("oak"), Some(String::from("oak")));
    sized.extend([String::from("elm"), String::from("fir")]);
    let extracted: std::collections::hash_set::ExtractIf<'_, _, _> =
        sized.extract_if(|key| key == "elm");
    assert_eq!(extracted.collect::<Vec<_>>(), [String::from("elm")]);
    sized.shrink_to(8);
    assert!(sized.capacity() >= 8);
    sized.shrink_to_fit();
    assert!(sized.capacity() >= sized.len());
    let mut cloned = HashSet::new();
    cloned.clone_from(&sized);
    assert_eq!(cloned, sized);
    let drained: std::collections::hash_set::Drain<'_, _> = sized.drain();
    assert_eq!(drained.collect::<Vec<_>>(), [String::from("fir")]);
    assert!(sized.is_empty());

    let left = HashSet::from([1, 2]);
    let right = HashSet::from([2, 3]);
    let union: std::collections::hash_set::Union<'_, _, _> = left.union(&right);
    assert_eq!(
        union.copied().collect::<HashSet<_>>(),
        HashSet::from([1, 2, 3])
    );
    let common: std::collections::hash_set::Intersection<'_, _, _> = left.intersection(&right);
    assert_eq!(common.copied().collect::<HashSet<_>>(), HashSet::from([2]));
    let difference: std::collections::hash_set::Difference<'_, _, _> = left.difference(&right);
    assert_eq!(
        difference.copied().collect::<HashSet<_>>(),
        HashSet::from([1])
    );
    let symmetric: std::collections::hash_set::SymmetricDifference<'_, _, _> =
        left.symmetric_difference(&right);
    assert_eq!(
        symmetric.copied().collect::<HashSet<_>>(),
        HashSet::from([1, 3])
    );
    assert_eq!(&left | &right, HashSet::from([1, 2, 3]));
    assert_eq!(&left & &right, HashSet::from([2]));
    assert_eq!(&left ^ &right, HashSet::from([1, 3]));
    assert_eq!(&left - &right, HashSet::from([1]));
    assert!(HashSet::from([1]).is_subset(&left));
    assert!(!left.is_subset(&right));
    assert!(left.is_superset(&HashSet::from([1])));
    assert!(!left.is_superset(&right));
    assert!(left.is_disjoint(&HashSet::from([3])));
    assert!(!left.is_disjoint(&right));
}

fn other_collections() {
    let mut map = BTreeMap::new();
    map.insert(2, 3);
    map.insert(1, 4);
    pure(&map);
    assert_eq!(map.into_iter().collect::<Vec<_>>(), [(1, 4), (2, 3)]);
    let mut set = BTreeSet::new();
    set.insert(2);
    set.insert(1);
    pure(&set);
    assert_eq!(set.into_iter().collect::<Vec<_>>(), [1, 2]);
    let mut queue = VecDeque::new();
    queue.push_back(2);
    queue.push_front(1);
    pure(&queue);
    assert_eq!(queue.pop_front(), Some(1));
    assert_eq!(queue.pop_back(), Some(2));
    let mut heap = BinaryHeap::new();
    heap.push(1);
    heap.push(2);
    pure(&heap);
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
}

#[tide::main]
fn main(root: Root) {
    maps();
    sets();
    other_collections();
    let state = RandomState::default();
    pure(&state);
    assert_eq!(state.hash_one("oak"), state.clone().hash_one("oak"));
    assert_eq!(
        state.hash_one("oak"),
        RandomState::default().hash_one("oak")
    );
    assert_eq!(format!("{state:?}"), "RandomState { .. }");
    let random = RandomState::from_rand(&root.rand);
    pure(&random);
    assert_eq!(format!("{random:?}"), "RandomState { .. }");
    assert_eq!(random.hash_one("oak"), random.clone().hash_one("oak"));
    eprintln!(
        "{} {} {}",
        random.hash_one(1u64),
        random.hash_one(2u64),
        random.hash_one(3u64)
    );
    let mut keyed = HashMap::with_hasher(random.clone());
    keyed.insert("oak", 7);
    assert_eq!(keyed["oak"], 7);
    pure(&keyed);
    let mut keyed_set = HashSet::with_hasher(random);
    keyed_set.insert("oak");
    assert!(keyed_set.contains("oak"));
    pure(&keyed_set);
    let mut sized_keyed = HashMap::with_capacity_and_hasher(8, keyed.hasher().clone());
    sized_keyed.insert("oak", 7);
    assert_eq!(sized_keyed["oak"], 7);
    let mut sized_keyed_set = HashSet::with_capacity_and_hasher(8, keyed_set.hasher().clone());
    sized_keyed_set.insert("oak");
    assert!(sized_keyed_set.contains("oak"));

    let mut map = HashMap::new();
    let mut set = HashSet::new();
    for n in 0..64 {
        map.insert(n, n + 1);
        set.insert(n);
    }
    let order: Vec<_> = map.iter().map(|(&k, &v)| (k, v)).collect();
    let set_order: Vec<_> = set.iter().copied().collect();
    let mut capacity = HashMap::with_capacity(0);
    let mut default = HashMap::default();
    let mut set_capacity = HashSet::with_capacity(0);
    let mut set_default = HashSet::default();
    for n in 0..64 {
        capacity.insert(n, n + 1);
        default.insert(n, n + 1);
        set_capacity.insert(n);
        set_default.insert(n);
    }
    let collected: HashMap<_, _> = (0..64).map(|n| (n, n + 1)).collect();
    let set_collected: HashSet<_> = (0..64).collect();
    for candidate in [capacity, default, collected] {
        assert_eq!(candidate, map);
        println!("{:?}", candidate.into_iter().collect::<Vec<_>>());
    }
    for candidate in [set_capacity, set_default, set_collected] {
        assert_eq!(candidate, set);
        println!("{:?}", candidate.into_iter().collect::<Vec<_>>());
    }
    assert_eq!(HashMap::from([(1, 2), (1, 3)])[&1], 3);
    assert_eq!(HashSet::from([1, 1]).len(), 1);
    // Nonzero capacity may change bucket order; it still executes and uses the fixed builder.
    assert!(HashMap::<i32, i32>::with_capacity(32).is_empty());
    assert!(HashSet::<i32>::with_capacity(32).is_empty());
    println!("{order:?}\n{set_order:?}");
}
