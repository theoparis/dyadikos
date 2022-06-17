use dyadikos_core::{mesh::Mesh, native::NativeApp, App, AppSettings};
use dyadikos_math::{transform::RenderTransformation, Vertex};
use glam::{Mat4, Vec3};
use wgpu::Color;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();

	let shader = r#"
	struct VertexOutput {
		@builtin(position) position: vec4<f32>,
	};

	@group(0)
	@binding(0)
	var<uniform> transform: mat4x4<f32>;

	@vertex
	fn vs_main(
		@location(0) position: vec3<f32>,
	) -> VertexOutput {
		var result: VertexOutput;
		result.position = transform * vec4<f32>(position, 1.0);
		return result;
	}

	@fragment
	fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
		return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	}
	"#;

	let app = NativeApp::new(AppSettings {
		shader: shader.to_string(),
		background_color: Color::BLACK,
		..Default::default()
	})
	.await?;

	let mut transform = RenderTransformation::default();
	transform.proj =
		Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.01, 1000.0);
	transform.view = Mat4::look_at_rh(
		Vec3::new(0.0, 0.0, 1.0),
		Vec3::new(0.0, 0.0, 0.0),
		Vec3::new(0.0, -1.0, 0.0),
	);
	transform.model = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));

	let matrix =
		(transform.proj * transform.view * transform.model).to_cols_array();

	let vertices = vec![
		Vertex {
			position: [0.5, 0.5, 0.0],
		},
		Vertex {
			position: [0.5, -0.5, 0.0],
		},
		Vertex {
			position: [-0.5, -0.5, 0.0],
		},
		Vertex {
			position: [-0.5, 0.5, 0.0],
		},
	];
	let indices = vec![0, 1, 3, 1, 2, 3];
	let mut mesh = Mesh::new(&app, vertices, indices);

	app.clone().run(
		&matrix,
		Box::new(move |rpass, uniform_buffer| {
			app.queue.write_buffer(
				uniform_buffer,
				0,
				bytemuck::cast_slice(&[matrix]),
			);
			mesh.render(rpass);
		}),
	);

	Ok(())
}
