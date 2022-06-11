use dyadikos::{mesh::Mesh, shader::load_shader, Vertex, VulkanApp};
use shaderc::ShaderKind;

fn main() {
	// Fixes the "InitializationFailed" error on wayland.
	std::env::set_var("WINIT_UNIX_BACKEND", "x11");
	tracing_subscriber::fmt::init();

	let mut app = VulkanApp::new(0).unwrap();

	let vertices = vec![
		Vertex {
			position: [-0.5, -0.25, 0.0],
		},
		Vertex {
			position: [0.0, 0.5, 0.0],
		},
		Vertex {
			position: [0.25, -0.1, 0.0],
		},
	];

	let mesh = Mesh::new(app.device.clone(), vertices).unwrap();

	let vertex_shader = load_shader(
		app.device.clone(),
		ShaderKind::Vertex,
		r#"
	#version 450
	layout(location = 0) in vec3 position;
	void main() {
			gl_Position = vec4(position, 1.0);
	}
	"#,
	)
	.unwrap();

	let fragment_shader = load_shader(
		app.device.clone(),
		ShaderKind::Fragment,
		r#"
	#version 450
	layout(location = 0) out vec4 f_color;
	void main() {
		f_color = vec4(1.0, 0.0, 0.0, 1.0);
	}
	"#,
	)
	.unwrap();

	app.run(
		vertex_shader,
		fragment_shader,
		Box::new(move |builder, framebuffer, viewport, pipeline| {
			mesh.draw(
				builder,
				framebuffer,
				viewport,
				pipeline,
				[0.0, 0.0, 1.0, 1.0].into(),
			)
			.unwrap();
		}),
	)
	.unwrap();
}
