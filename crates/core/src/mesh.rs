use crate::{App, ArcRenderPass};
use dyadikos_math::Vertex;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::Buffer;

pub struct Mesh {
	vertex_buffer: Arc<Buffer>,
	index_buffer: Arc<Buffer>,
	pub vertex_data: Vec<Vertex>,
	pub index_data: Vec<u32>,
}

impl Mesh {
	pub fn new(
		app: &impl App,
		vertex_data: Vec<Vertex>,
		index_data: Vec<u32>,
	) -> Self {
		let device = app.get_device();
		let vertex_buffer =
			device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(&vertex_data),
				usage: wgpu::BufferUsages::VERTEX,
			});

		let index_buffer =
			device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(&index_data),
				usage: wgpu::BufferUsages::INDEX,
			});

		Mesh {
			vertex_data,
			index_data,
			vertex_buffer: Arc::new(vertex_buffer),
			index_buffer: Arc::new(index_buffer),
		}
	}

	pub fn render(&mut self, mut rpass: ArcRenderPass) {
		rpass.set_vertex_buffer(0, self.vertex_buffer.clone());
		rpass.set_index_buffer(
			wgpu::IndexFormat::Uint32,
			self.index_buffer.clone(),
		);
		rpass.draw_indexed(0..self.index_data.len() as u32, 0, 0..1);
	}
}
