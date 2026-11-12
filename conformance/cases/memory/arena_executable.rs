//@ rules: ARENA-1, PURE-1, AFREE-1
// Shared production/conformance surface; tests/build.rs also executes this program.
use std::arena::{Arena, Id};
use std::collections::{BTreeSet, HashSet};
use std::fs::Fs;

struct Node { name: String, edges: Vec<Id<Node>> }

fn build() -> (Arena<Node>, Id<Node>) {
    let mut g = Arena::new();
    let a = g.insert(Node { name: "a".to_string(), edges: Vec::new() });
    let b = g.insert(Node { name: "b".to_string(), edges: vec![a] });
    g[a].edges.push(b);
    (g, a)
}

#[pure]
fn reachable(g: &Arena<Node>, start: Id<Node>) -> Vec<&str> {
    let (mut seen, mut stack, mut out) = (HashSet::new(), vec![start], Vec::new());
    while let Some(id) = stack.pop() {
        if !seen.insert(id) { continue; }
        out.push(g[id].name.as_str());
        stack.extend(g[id].edges.iter().copied());
    }
    out
}

#[pure]
fn copy_id<T>(id: Id<T>) -> Id<T> { id }
fn pure_value<T: Pure>(_: &T) {}

#[tide::main]
fn main(root: Root) {
    let (mut g, a) = build();
    assert_eq!(reachable(&g, a), vec!["a", "b"]);
    pure_value(&g);
    let b = g[a].edges[0];
    assert_eq!(g.len(), 2);
    assert!(g.contains(b));
    let [second, first] = g.get_disjoint_mut([b, a]).unwrap();
    second.name.push('2');
    first.name.push('1');
    assert_eq!(g[a].name, "a1");
    assert_eq!(g[b].name, "b2");
    assert!(g.get_disjoint_mut([a, a]).is_none());
    assert_eq!(g.get_disjoint_mut([]).unwrap().len(), 0);
    assert_eq!(g.remove(b).unwrap().name, "b2");
    assert!(g.get(b).is_none());
    assert!(g.get_mut(b).is_none());
    assert!(!g.contains(b));
    assert!(g.remove(b).is_none());
    assert!(g.get_disjoint_mut([a, b]).is_none());
    let c = g.insert(Node { name: "c".into(), edges: vec![] });
    assert_ne!(b, c);
    assert!(b < c);
    assert!(g.get(b).is_none());
    g.get_mut(c).unwrap().name.push('3');
    assert_eq!(g.iter().map(|(id, _)| id).collect::<Vec<_>>(), vec![a, c]);
    let mut ordered = BTreeSet::new();
    ordered.insert(c);
    ordered.insert(a.clone());
    assert_eq!(ordered.into_iter().collect::<Vec<_>>(), vec![a, c]);
    assert!(!format!("{a:?}").is_empty());
    let mut data = Arena::default();
    assert!(data.is_empty());
    let x = data.insert(10);
    let y = data.insert(20);
    let z = data.insert(30);
    data.remove(x);
    data.remove(z);
    let reused_z = data.insert(31);
    let reused_x = data.insert(11);
    assert_eq!(data.iter().map(|(id, _)| id).collect::<Vec<_>>(), vec![reused_x, y, reused_z]);
    assert_eq!(data.len(), 3);
    let mut capabilities = Arena::new();
    let id = capabilities.insert(root.fs);
    pure_value::<Id<Fs>>(&id);
    assert_eq!(copy_id(id), id);
    assert!(capabilities.contains(id));
    println!("ARENA_RAN");
}
