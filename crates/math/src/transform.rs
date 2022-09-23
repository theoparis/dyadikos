use nalgebra::{Matrix4, Vector3, Rotation3};

#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct ProjectionTransform {
	pub model: Matrix4<f32>,
	pub view: Matrix4<f32>,
	pub proj: Matrix4<f32>,
}

impl ProjectionTransform {
	pub fn default() -> Self {
		Self {
			model: Matrix4::identity(),
			view: Matrix4::identity(),
			proj: Matrix4::identity(),
		}
	}
}

#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct ObjectTransform {
	pub position: Vector3<f32>,
	pub rotation: Rotation3<f32>,
	pub scale: Vector3<f32>,
}

impl ObjectTransform {
	pub fn get_matrix(&self) -> Matrix4<f32> {
		let mut matrix = Matrix4::identity();
    matrix = matrix.append_translation(&self.position);
    matrix = matrix.append_nonuniform_scaling(&self.scale);
    matrix = self.rotation.to_homogeneous() * matrix;
    matrix
	}
}
