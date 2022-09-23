use crate::{App, ArcRenderPass};
use dyadikos_math::Vertex;
use nalgebra::Matrix4;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, Buffer};

pub struct Mesh {
	vertex_buffer: Arc<Buffer>,
	index_buffer: Arc<Buffer>,
	uniform_buffer: Arc<Buffer>,
	bind_group: Arc<BindGroup>,
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

		let uniform_buffer = app.get_device().create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Uniform Buffer"),
				contents: bytemuck::cast_slice(Matrix4::<f32>::identity().as_slice()),
				usage: wgpu::BufferUsages::UNIFORM
					| wgpu::BufferUsages::COPY_DST,
			},
		);

		let bind_group =
			app.get_device()
				.create_bind_group(&wgpu::BindGroupDescriptor {
					layout: &app.get_bind_group_layout(),
					entries: &[wgpu::BindGroupEntry {
						binding: 0,
						resource: uniform_buffer.as_entire_binding(),
					}],
					label: None,
				});

		Mesh {
			vertex_data,
			index_data,
			uniform_buffer: Arc::new(uniform_buffer),
			bind_group: Arc::new(bind_group),
			vertex_buffer: Arc::new(vertex_buffer),
			index_buffer: Arc::new(index_buffer),
		}
	}

	pub fn render<'pass>(
		&'pass mut self,
		mut rpass: ArcRenderPass<'pass>,
		app: &impl App,
		transform: &Matrix4<f32>,
	) {
		app.get_queue().write_buffer(
			&self.uniform_buffer,
			0,
			bytemuck::cast_slice(transform.as_slice()),
		);

		rpass.set_bind_group(0, &self.bind_group, &[]);

		rpass.set_vertex_buffer(0, self.vertex_buffer.clone());
		rpass.set_index_buffer(
			wgpu::IndexFormat::Uint32,
			self.index_buffer.clone(),
		);
		rpass.draw_indexed(0..self.index_data.len() as u32, 0, 0..1);
	}
}
