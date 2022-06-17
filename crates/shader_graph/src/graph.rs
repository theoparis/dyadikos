use petgraph::{
	algo, graph::NodeIndex, visit::EdgeRef, EdgeDirection, Graph as PetGraph,
	Incoming, Outgoing,
};
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum Dim {
	Dim1D = 0u32,
	Dim2D = 1u32,
	Dim3D = 2u32,
	DimCube = 3u32,
	DimRect = 4u32,
	DimBuffer = 5u32,
	DimSubpassData = 6u32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TypedValue {
	Float(f64),
	Vec2(f64, f64),
	Vec3(f64, f64, f64),
	Vec4(f64, f64, f64, f64),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeName {
	Bool,
	Int(bool),
	// single/double precision floating point type
	Float(bool),
	Vec(u32),
	Mat(u32, Box<TypeName>),
	Sampler(Box<TypeName>, Dim),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Node {
	Input(u32, Box<TypeName>),
	Uniform(u32, Box<TypeName>),
	Output(u32, Box<TypeName>),
	Constant(TypedValue),
	Construct(Box<TypeName>),
	Extract(u32),
	Normalize,
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulus,
	Clamp,
	Dot,
	Cross,
	Floor,
	Ceil,
	Round,
	Sin,
	Cos,
	Tan,
	Pow,
	Min,
	Max,
	Length,
	Distance,
	Reflect,
	Refract,
	Mix,
	Sample,
}

/// Convenience wrapper for [`petgraph::Graph`](petgraph::graph::Graph)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Graph {
	graph: PetGraph<Node, u32>,
}

impl Default for Graph {
	/// Create a new empty graph
	fn default() -> Self {
		Self {
			graph: PetGraph::new(),
		}
	}
}

impl Graph {
	/// Add a node to the graph
	pub fn add_node(&mut self, node: Node) -> NodeIndex<u32> {
		self.graph.add_node(node)
	}

	/// Add an edge between two nodes in the graph, infering the result type of the origin node
	pub fn add_edge(
		&mut self,
		from: NodeIndex<u32>,
		to: NodeIndex<u32>,
		index: u32,
	) {
		self.graph.add_edge(from, to, index);
	}

	pub fn has_cycle(&self) -> bool {
		algo::is_cyclic_directed(&self.graph)
	}

	/// List all the outputs of the graph
	pub fn outputs(&'_ self) -> impl Iterator<Item = NodeIndex<u32>> + '_ {
		self.graph.externals(Outgoing).filter(move |index| {
			matches!(self.graph.node_weight(*index), Some(&Node::Output(_, _)))
		})
	}

	pub fn arguments(
		&'_ self,
		index: NodeIndex<u32>,
	) -> impl Iterator<Item = NodeIndex<u32>> + '_ {
		let mut vec: Vec<_> =
			self.graph.edges_directed(index, Incoming).collect();

		vec.sort_by_key(|e| e.weight());

		vec.into_iter().map(|e| e.source())
	}

	pub fn neighbors(
		&self,
		index: NodeIndex<u32>,
		dir: Option<EdgeDirection>,
	) -> petgraph::graph::Neighbors<u32> {
		self.graph
			.neighbors_directed(index, dir.unwrap_or(EdgeDirection::Incoming))
	}
}

impl Index<NodeIndex<u32>> for Graph {
	type Output = Node;

	/// Get a node from the graph
	fn index(&self, index: NodeIndex<u32>) -> &Node {
		&self.graph[index]
	}
}
