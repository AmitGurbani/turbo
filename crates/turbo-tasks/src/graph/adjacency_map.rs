use std::collections::{HashMap, HashSet};

use super::graph_store::{GraphNode, GraphStore};

/// A graph traversal that builds an adjacency map
pub struct AdjacencyMap<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    adjacency_map: HashMap<T, Vec<T>>,
    roots: Vec<T>,
}

impl<T> Default for AdjacencyMap<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AdjacencyMap<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    /// Creates a new adjacency map
    pub fn new() -> Self {
        Self {
            adjacency_map: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Returns an iterator over the root nodes of the graph
    pub fn roots(&self) -> impl Iterator<Item = &T> {
        self.roots.iter()
    }

    /// Returns an iterator over the children of the given node
    pub fn get(&self, node: &T) -> Option<impl Iterator<Item = &T>> {
        self.adjacency_map.get(node).map(|vec| vec.iter())
    }
}

impl<T> GraphStore for AdjacencyMap<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    type Node = T;
    type Handle = T;

    fn insert(&mut self, from_handle: Option<T>, node: GraphNode<T>) -> Option<(Self::Handle, &T)> {
        let vec = if let Some(from_handle) = from_handle {
            self.adjacency_map
                .entry(from_handle)
                .or_insert_with(|| Vec::with_capacity(1))
        } else {
            &mut self.roots
        };

        vec.push(node.node().clone());
        Some((node.into_node(), vec.last().unwrap()))
    }
}

impl<T> AdjacencyMap<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    /// Returns an iterator over the nodes in reverse topological order,
    /// starting from the roots.
    pub fn into_reverse_topological(self) -> ReverseTopologicalIter<T> {
        ReverseTopologicalIter {
            adjacency_map: self.adjacency_map,
            stack: self
                .roots
                .into_iter()
                .map(|root| (ReverseTopologicalPass::Pre, root))
                .collect(),
            visited: HashSet::new(),
        }
    }

    /// Returns an iterator over the nodes in reverse topological order,
    /// starting from the given node.
    pub fn reverse_topological_from_node<'graph>(
        &'graph self,
        node: &'graph T,
    ) -> ReverseTopologicalFromNodeIter<'graph, T> {
        ReverseTopologicalFromNodeIter {
            adjacency_map: &self.adjacency_map,
            stack: vec![(ReverseTopologicalPass::Pre, node)],
            visited: HashSet::new(),
        }
    }
}

#[derive(Debug)]
enum ReverseTopologicalPass {
    Pre,
    Post,
}

/// An iterator over the nodes of a graph in reverse topological order, starting
/// from the roots.
pub struct ReverseTopologicalIter<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    adjacency_map: HashMap<T, Vec<T>>,
    stack: Vec<(ReverseTopologicalPass, T)>,
    visited: HashSet<T>,
}

impl<T> Iterator for ReverseTopologicalIter<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = loop {
            let (pass, current) = self.stack.pop()?;

            match pass {
                ReverseTopologicalPass::Post => {
                    break current;
                }
                ReverseTopologicalPass::Pre => {
                    if self.visited.contains(&current) {
                        continue;
                    }

                    self.visited.insert(current.clone());

                    let Some(neighbors) = self.adjacency_map.get(&current) else {
                        break current;
                    };

                    self.stack.push((ReverseTopologicalPass::Post, current));
                    self.stack.extend(
                        neighbors
                            .iter()
                            .map(|neighbor| (ReverseTopologicalPass::Pre, neighbor.clone())),
                    );
                }
            }
        };

        Some(current)
    }
}

/// An iterator over the nodes of a graph in reverse topological order, starting
/// from a given node.
pub struct ReverseTopologicalFromNodeIter<'graph, T>
where
    T: Eq + std::hash::Hash + Clone,
{
    adjacency_map: &'graph HashMap<T, Vec<T>>,
    stack: Vec<(ReverseTopologicalPass, &'graph T)>,
    visited: HashSet<&'graph T>,
}

impl<'graph, T> Iterator for ReverseTopologicalFromNodeIter<'graph, T>
where
    T: Eq + std::hash::Hash + Clone,
{
    type Item = &'graph T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = loop {
            let (pass, current) = self.stack.pop()?;

            match pass {
                ReverseTopologicalPass::Post => {
                    break current;
                }
                ReverseTopologicalPass::Pre => {
                    if self.visited.contains(&current) {
                        continue;
                    }

                    self.visited.insert(current);

                    let Some(neighbors) = self.adjacency_map.get(current) else {
                        break current;
                    };

                    self.stack.push((ReverseTopologicalPass::Post, current));
                    self.stack.extend(
                        neighbors
                            .iter()
                            .map(|neighbor| (ReverseTopologicalPass::Pre, neighbor)),
                    );
                }
            }
        };

        Some(current)
    }
}
