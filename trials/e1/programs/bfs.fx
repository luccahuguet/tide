// src/graph.fx of a library package
//
// A directed graph stored in an arena. Nodes refer to each other by
// generational handles, so cycles need no `Rc` and a removed node's handle
// goes stale instead of dangling.

use std::arena::{Arena, Handle};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Node {
    pub name: String,
    edges: Vec<Handle<Node>>,
}

pub struct Graph {
    nodes: Arena<Node>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph { nodes: Arena::new() }
    }

    pub fn add_node(&mut self, name: &str) -> Handle<Node> {
        self.nodes.insert(Node { name: name.to_string(), edges: Vec::new() })
    }

    /// Adds the edge `from -> to`. Panics if either handle is stale.
    pub fn add_edge(&mut self, from: Handle<Node>, to: Handle<Node>) {
        assert!(self.nodes.get(to).is_some(), "stale edge target");
        self.nodes[from].edges.push(to);
    }

    /// Removes a node. Edges that point at it go stale and are skipped by searches.
    pub fn remove_node(&mut self, h: Handle<Node>) -> Option<Node> {
        self.nodes.remove(h)
    }

    pub fn name(&self, h: Handle<Node>) -> Option<&str> {
        self.nodes.get(h).map(|n| n.name.as_str())
    }

    /// Authority-free. A shortest path from `start` to `goal` by edge count,
    /// both ends included, or `None` if `goal` is unreachable or either handle
    /// is stale.
    pub fn shortest_path(&self, start: Handle<Node>, goal: Handle<Node>) -> Option<Vec<Handle<Node>>> {
        self.nodes.get(start)?;
        self.nodes.get(goal)?;

        let mut parent: HashMap<Handle<Node>, Handle<Node>> = HashMap::new();
        let mut seen: HashSet<Handle<Node>> = HashSet::from([start]);
        let mut queue: VecDeque<Handle<Node>> = VecDeque::from([start]);

        while let Some(h) = queue.pop_front() {
            if h == goal {
                let mut path = vec![goal];
                let mut cur = goal;
                while let Some(&p) = parent.get(&cur) {
                    path.push(p);
                    cur = p;
                }
                path.reverse();
                return Some(path);
            }
            for &next in &self.nodes[h].edges {
                // Skip edges to removed nodes; `insert` is false for nodes already seen.
                if self.nodes.get(next).is_some() && seen.insert(next) {
                    parent.insert(next, h);
                    queue.push_back(next);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(g: &Graph, path: &[Handle<Node>]) -> Vec<String> {
        path.iter().map(|&h| g.name(h).unwrap().to_string()).collect()
    }

    /// a -> b -> c -> e (long), a -> d -> e (short), plus a back edge e -> a.
    fn sample() -> (Graph, [Handle<Node>; 5]) {
        let mut g = Graph::new();
        let hs = ["a", "b", "c", "d", "e"].map(|n| g.add_node(n));
        let [a, b, c, d, e] = hs;
        g.add_edge(a, b);
        g.add_edge(b, c);
        g.add_edge(c, e);
        g.add_edge(a, d);
        g.add_edge(d, e);
        g.add_edge(e, a);
        (g, hs)
    }

    #[test]
    fn finds_the_shortest_route() {
        let (g, [a, _, _, _, e]) = sample();
        let path = g.shortest_path(a, e).unwrap();
        assert_eq!(names(&g, &path), ["a", "d", "e"]);
    }

    #[test]
    fn path_to_self_is_one_node() {
        let (g, [a, ..]) = sample();
        assert_eq!(g.shortest_path(a, a), Some(vec![a]));
    }

    #[test]
    fn edges_are_directed() {
        let (g, [_, b, _, d, _]) = sample();
        // b -> c -> e -> a -> d
        assert_eq!(names(&g, &g.shortest_path(b, d).unwrap()), ["b", "c", "e", "a", "d"]);
        let mut g = g;
        let lonely = g.add_node("lonely");
        assert_eq!(g.shortest_path(b, lonely), None);
    }

    #[test]
    fn removed_nodes_are_routed_around() {
        let (mut g, [a, _, _, d, e]) = sample();
        g.remove_node(d).unwrap();
        assert_eq!(names(&g, &g.shortest_path(a, e).unwrap()), ["a", "b", "c", "e"]);
        assert_eq!(g.shortest_path(a, d), None); // stale goal
    }
}
