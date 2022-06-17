use dyadikos_math::Matrix4;
use std::{ops::Range, sync::Arc};
use typed_arena::Arena;
use wgpu::{
	BindGroup, Buffer, Color, Device, DynamicOffset, Features, IndexFormat,
	PrimitiveState, RenderPass, RenderPipeline,
};

pub type RenderCallback = dyn FnMut(ArcRenderPass, &mut Buffer);

#[derive(Debug, Clone, Default)]
pub struct AppSettings {
	pub primitive_state: PrimitiveState,
	pub shader: String,
	pub features: Features,
	pub background_color: Color,
}

pub trait App {
	fn get_window_size(&self) -> (u32, u32);
	fn get_settings(&self) -> &AppSettings;
	fn get_device(&self) -> &Device;
	fn get_pipeline(&self) -> &RenderPipeline;
	fn get_bind_group(&self) -> &BindGroup;
	fn run(self, matrix: &Matrix4, callback: Box<RenderCallback>);
}

pub struct ArcRenderPass<'a> {
	arena: &'a Arena<Arc<Buffer>>,
	render_pass: RenderPass<'a>,
}

impl<'a> ArcRenderPass<'a> {
	pub fn set_vertex_buffer(&mut self, slot: u32, buffer: Arc<Buffer>) {
		let buffer = self.arena.alloc(buffer);
		self.render_pass.set_vertex_buffer(slot, buffer.slice(..));
	}

	pub fn set_index_buffer(
		&mut self,
		format: IndexFormat,
		buffer: Arc<Buffer>,
	) {
		let buffer = self.arena.alloc(buffer);
		self.render_pass.set_index_buffer(buffer.slice(..), format);
	}

	pub fn draw_indexed(
		&mut self,
		indices: Range<u32>,
		base_vertex: i32,
		instances: Range<u32>,
	) {
		self.render_pass
			.draw_indexed(indices, base_vertex, instances)
	}

	pub fn set_bind_group(
		&mut self,
		slot: u32,
		bind_group: &'a Arc<BindGroup>,
		offsets: &[DynamicOffset],
	) {
		self.render_pass.set_bind_group(slot, bind_group, offsets);
	}
}
pub mod mesh;

#[cfg(not(target_arch = "wasm"))]
pub mod native;
