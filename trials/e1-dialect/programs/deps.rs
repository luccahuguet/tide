// Task 9: deps -- src/lib.rs
//
// A dependency graph in an `Arena`: handles are generation-checked ids, so a
// handle to a removed node reads as stale, never as whatever reuses its slot.

use std::arena::{Arena, Id};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug)]
struct Node {
    name: String,
    needs: Vec<Id<Node>>, // distinct, live
}

/// A node of a `Graph`; stale once that node is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(Id<Node>);

#[derive(Debug, Error, PartialEq)]
pub enum GraphError {
    #[error("handle refers to a removed node")]
    Stale,
    #[error("dependency cycle: {0:?}")]
    Cycle(Vec<String>),
}

pub struct Graph {
    nodes: Arena<Node>,
    by_name: HashMap<String, Id<Node>>,
    inserted: Vec<Id<Node>>, // live nodes in insertion order
}

impl Graph {
    pub fn new() -> Graph {
        Graph { nodes: Arena::new(), by_name: HashMap::new(), inserted: Vec::new() }
    }

    /// Adds a node named `name`, or returns the existing node of that name.
    pub fn add(&mut self, name: &str) -> Handle {
        if let Some(&id) = self.by_name.get(name) {
            return Handle(id);
        }
        let id = self.nodes.insert(Node { name: name.to_string(), needs: Vec::new() });
        self.by_name.insert(name.to_string(), id);
        self.inserted.push(id);
        Handle(id)
    }

    /// The node's name, or `None` for a stale handle.
    pub fn name(&self, h: Handle) -> Option<&str> {
        self.nodes.get(h.0).map(|n| n.name.as_str())
    }

    /// Records that `a` needs `b`.
    pub fn depends(&mut self, a: Handle, b: Handle) -> Result<(), GraphError> {
        if self.nodes.get(b.0).is_none() {
            return Err(GraphError::Stale);
        }
        let node = self.nodes.get_mut(a.0).ok_or(GraphError::Stale)?;
        if !node.needs.contains(&b.0) {
            node.needs.push(b.0);
        }
        Ok(())
    }

    /// Deletes a node and every edge to it; returns its name.
    pub fn remove(&mut self, h: Handle) -> Result<String, GraphError> {
        let node = self.nodes.remove(h.0).ok_or(GraphError::Stale)?;
        self.by_name.remove(&node.name);
        self.inserted.retain(|&id| id != h.0);
        for &id in &self.inserted {
            self.nodes[id].needs.retain(|&dep| dep != h.0);
        }
        Ok(node.name)
    }

    /// Names in an order where every node follows its dependencies; among
    /// nodes that are ready, the earliest inserted comes first. Fails with
    /// the names of one cycle, in dependency order.
    pub fn order(&self) -> Result<Vec<&str>, GraphError> {
        let n = self.inserted.len();
        let pos: HashMap<Id<Node>, usize> =
            self.inserted.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let mut waiting = vec![0usize; n]; // dependencies not yet emitted
        let mut users: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (i, &id) in self.inserted.iter().enumerate() {
            for dep in &self.nodes[id].needs {
                waiting[i] += 1;
                users[pos[dep]].push(i);
            }
        }
        let mut ready: BTreeSet<usize> = (0..n).filter(|&i| waiting[i] == 0).collect();
        let mut done = vec![false; n];
        let mut out = Vec::with_capacity(n);
        while let Some(i) = ready.pop_first() {
            done[i] = true;
            out.push(self.nodes[self.inserted[i]].name.as_str());
            for &u in &users[i] {
                waiting[u] -= 1;
                if waiting[u] == 0 {
                    ready.insert(u);
                }
            }
        }
        if out.len() == n {
            Ok(out)
        } else {
            Err(GraphError::Cycle(self.find_cycle(&pos, &done)))
        }
    }

    /// Every node not `done` has a dependency not `done`, so following such
    /// edges from the earliest such node must revisit a node: that loop is a
    /// cycle.
    fn find_cycle(&self, pos: &HashMap<Id<Node>, usize>, done: &[bool]) -> Vec<String> {
        let mut seen_at: Vec<Option<usize>> = vec![None; done.len()];
        let mut path: Vec<usize> = Vec::new();
        let mut cur = done.iter().position(|&d| !d).expect("a node was left out");
        while seen_at[cur].is_none() {
            seen_at[cur] = Some(path.len());
            path.push(cur);
            cur = self.nodes[self.inserted[cur]]
                .needs
                .iter()
                .map(|dep| pos[dep])
                .find(|&j| !done[j])
                .expect("a node left out has a dependency left out");
        }
        let start = seen_at[cur].expect("the walk stops at a seen node");
        path[start..].iter().map(|&i| self.nodes[self.inserted[i]].name.clone()).collect()
    }

    /// The names of the nodes that directly need `h`, in insertion order
    /// (none for a stale handle).
    pub fn dependents(&self, h: Handle) -> impl Iterator<Item = &str> {
        self.inserted
            .iter()
            .map(move |&id| &self.nodes[id])
            .filter(move |node| node.needs.contains(&h.0))
            .map(|node| node.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond() {
        let mut g = Graph::new();
        let app = g.add("app");
        let left = g.add("left");
        let right = g.add("right");
        let base = g.add("base");
        g.depends(app, left).unwrap();
        g.depends(app, right).unwrap();
        g.depends(left, base).unwrap();
        g.depends(right, base).unwrap();
        assert_eq!(g.add("left"), left); // names are unique
        assert_eq!(g.order(), Ok(vec!["base", "left", "right", "app"]));
        assert_eq!(g.dependents(base).collect::<Vec<_>>(), vec!["left", "right"]);
        assert_eq!(g.dependents(left).collect::<Vec<_>>(), vec!["app"]);
        assert_eq!(g.dependents(app).count(), 0);
    }

    #[test]
    fn independent_nodes_keep_insertion_order() {
        let mut g = Graph::new();
        let c = g.add("c");
        g.add("a");
        let b = g.add("b");
        g.depends(c, b).unwrap();
        assert_eq!(g.order(), Ok(vec!["a", "b", "c"]));
    }

    #[test]
    fn cycle_is_named() {
        let mut g = Graph::new();
        let solo = g.add("solo");
        let x = g.add("x");
        let y = g.add("y");
        let z = g.add("z");
        g.depends(x, y).unwrap();
        g.depends(y, z).unwrap();
        g.depends(z, x).unwrap();
        g.depends(solo, solo).unwrap(); // a self-loop is a cycle too, and comes first
        assert_eq!(g.order(), Err(GraphError::Cycle(vec!["solo".to_string()])));
        g.remove(solo).unwrap();
        let want: Vec<String> = vec!["x".into(), "y".into(), "z".into()];
        assert_eq!(g.order(), Err(GraphError::Cycle(want)));
    }

    #[test]
    fn removing_a_node_breaks_a_cycle() {
        let mut g = Graph::new();
        let x = g.add("x");
        let y = g.add("y");
        let z = g.add("z");
        g.depends(x, y).unwrap();
        g.depends(y, z).unwrap();
        g.depends(z, x).unwrap();
        assert!(matches!(g.order(), Err(GraphError::Cycle(_))));
        assert_eq!(g.remove(z), Ok("z".to_string()));
        assert_eq!(g.order(), Ok(vec!["y", "x"]));
    }

    #[test]
    fn a_stale_handle_is_detected() {
        let mut g = Graph::new();
        let a = g.add("a");
        let b = g.add("b");
        g.depends(a, b).unwrap();
        assert_eq!(g.remove(b), Ok("b".to_string()));
        let c = g.add("c"); // may reuse b's slot, with a new generation
        assert_ne!(b, c);
        assert_eq!(g.name(b), None);
        assert_eq!(g.name(c), Some("c"));
        assert_eq!(g.depends(a, b), Err(GraphError::Stale));
        assert_eq!(g.depends(b, a), Err(GraphError::Stale));
        assert_eq!(g.remove(b), Err(GraphError::Stale));
        assert_eq!(g.dependents(b).count(), 0);
        assert_eq!(g.order(), Ok(vec!["a", "c"])); // the edge to b went with it
        let b2 = g.add("b");
        assert_ne!(b2, b);
    }
}
