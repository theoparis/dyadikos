use crate::Vertex;
use anyhow::Result;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
	command_buffer::{
		AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SubpassContents,
	},
	device::Device,
	format::ClearValue,
	pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
	render_pass::Framebuffer,
};

pub struct Mesh {
	vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
}

impl Mesh {
	pub fn new(device: Arc<Device>, vertices: Vec<Vertex>) -> Result<Self> {
		let vertex_buffer = CpuAccessibleBuffer::from_iter(
			device,
			BufferUsage::all(),
			false,
			vertices,
		)?;

		let mesh = Mesh { vertex_buffer };

		Ok(mesh)
	}

	pub fn draw(
		&self,
		builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
		framebuffer: Arc<Framebuffer>,
		viewport: Viewport,
		pipeline: Arc<GraphicsPipeline>,
		clear_values: ClearValue,
	) -> Result<()> {
		builder
			.begin_render_pass(
				framebuffer,
				SubpassContents::Inline,
				vec![clear_values],
			)?
			.set_viewport(0, [viewport])
			.bind_pipeline_graphics(pipeline)
			.bind_vertex_buffers(0, self.vertex_buffer.clone())
			.draw(self.vertex_buffer.len() as u32, 1, 0, 0)?
			.end_render_pass()?;

		Ok(())
	}
}
