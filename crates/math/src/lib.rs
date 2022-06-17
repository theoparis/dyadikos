use bytemuck::{Pod, Zeroable};

pub type Matrix4 = [f32; 16];
pub type Vector3 = [f32; 3];

pub fn identity() -> Matrix4 {
	[
		1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
		0.0, 1.0,
	]
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex {
	pub position: Vector3,
}

pub mod transform;
