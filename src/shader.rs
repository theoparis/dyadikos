use anyhow::Result;
use shaderc::ShaderKind;
use std::sync::Arc;
use tracing::debug;
use vulkano::{device::Device, shader::ShaderModule};

pub fn load_shader(
	device: Arc<Device>,
	kind: ShaderKind,
	source: &str,
) -> Result<Arc<ShaderModule>> {
	debug!("Loading {:?} shader", kind);
	let mut compiler = shaderc::Compiler::new().unwrap();
	let mut options = shaderc::CompileOptions::new().unwrap();
	options.add_macro_definition("EP", Some("main"));
	let binary_result = compiler.compile_into_spirv(
		source,
		kind,
		"shader.glsl",
		"main",
		Some(&options),
	)?;

	let binary_result = binary_result.as_binary();

	unsafe {
		let module = ShaderModule::from_words(device, binary_result)?;

		Ok(module)
	}
}
