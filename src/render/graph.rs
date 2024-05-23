
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(usize);

pub struct InGraph<T, E> {
    // NodeId = index
    nodes: Vec<T>,
    // from e.0 to index
    edges: Vec<Vec<(NodeId, E)>>,
}

#[allow(unused)]
impl<T, E> InGraph<T, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: T) -> NodeId {
        self.nodes.push(node);
        self.edges.push(Vec::new());
        NodeId(self.nodes.len()-1)
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge: E) {
        self.edges[to.0].push((NodeId(from.0), edge));
    }

    pub fn edges_to(&self, node: NodeId) -> Vec<(&T, &E)> {
        let mut edges = Vec::new();
        for (from, edge) in &self.edges[node.0] {
            edges.push((&self.nodes[from.0], edge));
        }
        edges.into()
    }

    pub fn edges_to_mut(&mut self, node: NodeId) -> Vec<(&T, &mut E)> {
        let mut edges = Vec::new();
        for (from, edge) in &mut self.edges[node.0] {
            edges.push((&self.nodes[from.0], edge));
        }
        edges.into()
    }

    pub fn map_nodes<U, F: Fn(T, &[(NodeId, E)]) -> U>(self, f: F) -> InGraph<U, E> {
        InGraph {
            nodes: self.nodes.into_iter().zip(&self.edges).map(|(n,e)| f(n, e)).collect(),
            edges: self.edges,
        }
    }

    pub fn map_edges<D, G: Fn(&T, &T, E) -> D>(self, g: G) -> InGraph<T, D> {
        let mut new_edges = vec![];
        
        for (to, edges) in self.nodes.iter().zip(self.edges) {
            let mut node_edges = vec![];
            for (from, edge) in edges {
                node_edges.push((from, g(&self.nodes[from.0], &to, edge)));
            }
            new_edges.push(node_edges);
        }
        
        InGraph {
            nodes: self.nodes,
            edges: new_edges,
        }
    }

    pub fn node(&self, id: NodeId) -> &T {
        &self.nodes[id.0]
    }

    pub fn nodes(&self) -> &[T] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [T] {
        &mut self.nodes
    }

    pub fn node_ids(&self) -> Vec<NodeId> {
        (0..self.nodes.len()).into_iter().map(|i| NodeId(i)).collect::<Vec<_>>().into()
    }

    pub fn topo_sort(&self) -> Vec<NodeId> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Mark {
            None,
            Temporary,
            Permanent,
        }

        let mut out = Vec::with_capacity(self.nodes.len());
        let mut marks = vec![Mark::None; self.nodes.len()];

        while marks.iter().filter(|&&m| m != Mark::Permanent).count() > 0 {
            let (n_idx, &n_mark) = marks.iter().enumerate().find(|(_, &m)| m == Mark::None).expect("has a cycle");

            visit(n_idx, n_mark, &mut marks, &self.edges, &mut out);

            fn visit<E>(n_idx: usize, n_mark: Mark, marks: &mut Vec<Mark>, edges: &Vec<Vec<(NodeId, E)>>, out: &mut Vec<NodeId>) {
                if n_mark == Mark::Permanent { return }
                if n_mark == Mark::Temporary { panic!("cycle detected") }

                marks[n_idx] = Mark::Temporary;

                for (m, _) in &edges[n_idx] {
                    visit(m.0, marks[m.0], marks, edges, out);
                }

                marks[n_idx] = Mark::Permanent;
                out.push(NodeId(n_idx));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test() {
        //   A   B
        //  / \   \
        // C   D   E
        //  \   \ /
        //   F-->G
        let mut graph = InGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());
        graph.add_edge(a, c, ());
        graph.add_edge(a, d, ());
        graph.add_edge(b, e, ());
        graph.add_edge(c, f, ());
        graph.add_edge(d, g, ());
        graph.add_edge(e, g, ());
        graph.add_edge(f, g, ());
        
        assert_eq!(graph.topo_sort(), [a, b, c, d, e, f, g]);
    }
}
