use glam::{Mat4, Quat, Vec3};

#[derive(PartialEq, Copy, Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RenderTransformation {
	pub model: Mat4,
	pub view: Mat4,
	pub proj: Mat4,
}

impl RenderTransformation {
	pub fn default() -> Self {
		Self {
			model: Mat4::IDENTITY,
			view: Mat4::IDENTITY,
			proj: Mat4::IDENTITY,
		}
	}
}

#[derive(PartialEq, Copy, Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct ObjectTransform {
	pub position: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}

impl ObjectTransform {
	pub fn get_matrix(&self) -> Mat4 {
		Mat4::from_scale_rotation_translation(
			self.scale,
			self.rotation,
			self.scale,
		)
	}
}
