//! Shortest paths by breadth-first search in an arena-backed directed graph.

use std::arena::{Arena, Id};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub edges: Vec<Id<Node>>,
}

/// A directed graph. Nodes live in an arena and point at each other through
/// `Id` handles, so cycles need neither `Rc` nor borrows.
pub struct Graph {
    nodes: Arena<Node>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph { nodes: Arena::new() }
    }

    pub fn add_node(&mut self, name: &str) -> Id<Node> {
        self.nodes.insert(Node { name: name.to_string(), edges: Vec::new() })
    }

    /// Adds the edge `from -> to`. Panics if `from` is stale.
    pub fn add_edge(&mut self, from: Id<Node>, to: Id<Node>) {
        self.nodes[from].edges.push(to);
    }

    /// Removes a node. Edges into it become stale and are ignored.
    pub fn remove_node(&mut self, id: Id<Node>) -> Option<Node> {
        self.nodes.remove(id)
    }

    pub fn node(&self, id: Id<Node>) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn has_edge(&self, from: Id<Node>, to: Id<Node>) -> bool {
        self.nodes.get(to).is_some() && self.nodes.get(from).is_some_and(|n| n.edges.contains(&to))
    }

    /// A path with the fewest edges from `start` to `goal`, both included
    /// (`[start]` when they are equal). `None` if `goal` is unreachable or
    /// either handle is stale.
    pub fn shortest_path(&self, start: Id<Node>, goal: Id<Node>) -> Option<Vec<Id<Node>>>
        ensures match &result {
            Some(path) => path.first() == Some(&start)
                && path.last() == Some(&goal)
                && path.windows(2).all(|w| self.has_edge(w[0], w[1])),
            None => true,
        }
    {
        if self.nodes.get(start).is_none() || self.nodes.get(goal).is_none() {
            return None;
        }
        let mut came_from: HashMap<Id<Node>, Id<Node>> = HashMap::new();
        let mut seen = HashSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(id) = queue.pop_front() {
            if id == goal {
                let mut path = vec![goal];
                let mut at = goal;
                while let Some(&prev) = came_from.get(&at) {
                    path.push(prev);
                    at = prev;
                }
                path.reverse();
                return Some(path);
            }
            for &next in &self.nodes[id].edges {
                if self.nodes.get(next).is_some() && seen.insert(next) {
                    came_from.insert(next, id);
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

    fn names(g: &Graph, path: &[Id<Node>]) -> Vec<String> {
        path.iter().map(|&id| g.node(id).unwrap().name.clone()).collect()
    }

    #[test]
    fn finds_the_shortest_of_several_paths() {
        let mut g = Graph::new();
        let [a, b, c, d, e] = ["a", "b", "c", "d", "e"].map(|name| g.add_node(name));
        for (from, to) in [(a, b), (b, c), (c, d), (a, e), (e, d), (d, a)] {
            g.add_edge(from, to);
        }
        let path = g.shortest_path(a, d).unwrap();
        assert_eq!(names(&g, &path), ["a", "e", "d"]);
        assert_eq!(g.shortest_path(d, d), Some(vec![d]));
        assert_eq!(g.shortest_path(d, c), Some(vec![d, a, b, c]));   // through the cycle
    }

    #[test]
    fn unreachable_and_stale_handles_give_none() {
        let mut g = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.add_edge(a, b);
        g.add_edge(b, c);
        assert_eq!(g.shortest_path(c, a), None);        // edges are directed
        assert!(g.remove_node(b).is_some());
        assert_eq!(g.shortest_path(a, c), None);        // the only route went through `b`
        assert_eq!(g.shortest_path(b, c), None);        // stale start
    }

    /// BFS distances from node 0 match brute-force relaxation on random graphs.
    #[test]
    fn paths_are_shortest(n: u8, edges: Vec<(u8, u8)>)
        requires n > 0
    {
        let n = usize::from(n);
        let edges: Vec<(usize, usize)> = edges.iter()
            .map(|&(a, b)| (usize::from(a) % n, usize::from(b) % n))
            .collect();
        let mut g = Graph::new();
        let ids: Vec<Id<Node>> = (0..n).map(|i| g.add_node(&i.to_string())).collect();
        for &(a, b) in &edges {
            g.add_edge(ids[a], ids[b]);
        }
        let mut dist: Vec<Option<usize>> = vec![None; n];
        dist[0] = Some(0);
        for _ in 0..n {
            for &(a, b) in &edges {
                if let Some(d) = dist[a] {
                    if dist[b].map_or(true, |old| d + 1 < old) {
                        dist[b] = Some(d + 1);
                    }
                }
            }
        }
        for v in 0..n {
            let got = g.shortest_path(ids[0], ids[v]).map(|path| path.len() - 1);
            assert_eq!(got, dist[v], "node {v}");
        }
    }
}
