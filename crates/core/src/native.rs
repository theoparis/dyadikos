use crate::{App, AppSettings, ArcRenderPass, RenderCallback};
use anyhow::{Context, Result};
use dyadikos_math::{Matrix4, Vertex};
use std::{
	borrow::Cow,
	sync::{Arc, Mutex, RwLock},
};
use typed_arena::Arena;
use wgpu::{
	util::DeviceExt, Backends, BindGroup, BindGroupLayout,
	CommandEncoderDescriptor, Device, DeviceDescriptor, FragmentState,
	Instance, Limits, LoadOp, MultisampleState, Operations,
	PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState,
	Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
	RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
	ShaderSource, Surface, SurfaceConfiguration, TextureUsages,
	TextureViewDescriptor, VertexState,
};
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::run_return::EventLoopExtRunReturn,
	window::Window,
};

#[derive(Clone)]
pub struct NativeApp {
	pub event_loop: Arc<RwLock<EventLoop<()>>>,
	pub window: Arc<Window>,
	pub surface: Arc<Surface>,
	pub device: Arc<Device>,
	pub config: Arc<Mutex<SurfaceConfiguration>>,
	pub queue: Arc<Queue>,
	pub settings: AppSettings,
	pub render_pipeline: Arc<RenderPipeline>,
	pub bind_group: Option<Arc<BindGroup>>,
	pub bind_group_layout: Arc<BindGroupLayout>,
}

impl App for NativeApp {
	fn get_settings(&self) -> &AppSettings {
		&self.settings
	}

	fn get_device(&self) -> &Device {
		&self.device
	}

	fn get_pipeline(&self) -> &RenderPipeline {
		&self.render_pipeline
	}

	fn get_bind_group(&self) -> &BindGroup {
		self.bind_group.as_ref().unwrap()
	}

	fn get_window_size(&self) -> (u32, u32) {
		let size = self.window.inner_size();

		(size.width, size.height)
	}

	fn run(mut self, matrix: &Matrix4, mut callback: Box<RenderCallback>) {
		let mut uniform_buffer =
			self.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Uniform Buffer"),
					contents: bytemuck::cast_slice(matrix),
					usage: wgpu::BufferUsages::UNIFORM
						| wgpu::BufferUsages::COPY_DST,
				});

		self.event_loop.try_write().unwrap().run_return(
			move |event, _, control_flow| {
				let config = self.config.clone();
				let mut config = config.try_lock().unwrap();
				let surface = self.surface.clone();
				let window = self.window.clone();
				let device = self.device.clone();

				*control_flow = ControlFlow::Wait;
				match event {
					Event::WindowEvent {
						event: WindowEvent::Resized(size),
						..
					} => {
						// Reconfigure the surface with the new size
						config.width = size.width;
						config.height = size.height;
						surface.configure(&device, &config);
						// On macos the window needs to be redrawn manually after resizing
						window.request_redraw();
					}
					Event::RedrawRequested(_) => {
						let frame = self
							.surface
							.get_current_texture()
							.context(
								"Failed to acquire next swap chain texture",
							)
							.unwrap();
						let view = frame
							.texture
							.create_view(&TextureViewDescriptor::default());

						self.bind_group =
							Some(Arc::new(device.create_bind_group(
								&wgpu::BindGroupDescriptor {
									layout: &self.bind_group_layout,
									entries: &[wgpu::BindGroupEntry {
										binding: 0,
										resource:
											uniform_buffer.as_entire_binding(),
									}],
									label: None,
								},
							)));

						let mut encoder = self.device.create_command_encoder(
							&CommandEncoderDescriptor { label: None },
						);
						{
							let mut rpass = encoder.begin_render_pass(
								&RenderPassDescriptor {
									label: None,
									color_attachments: &[Some(
										RenderPassColorAttachment {
											view: &view,
											resolve_target: None,
											ops: Operations {
												load: LoadOp::Clear(
													self.settings
														.background_color,
												),
												store: true,
											},
										},
									)],
									depth_stencil_attachment: None,
								},
							);
							rpass.set_pipeline(&self.render_pipeline);

							let mut rpass = ArcRenderPass {
								arena: &Arena::new(),
								render_pass: rpass,
							};
							rpass.set_bind_group(
								0,
								self.bind_group.as_ref().unwrap(),
								&[],
							);

							callback(rpass, &mut uniform_buffer);
						}

						self.queue.submit(Some(encoder.finish()));
						frame.present();
					}
					Event::WindowEvent {
						event: WindowEvent::CloseRequested,
						..
					} => *control_flow = ControlFlow::Exit,
					_ => {}
				}
			},
		);
	}
}

impl NativeApp {
	pub async fn new(settings: AppSettings) -> Result<Self> {
		let event_loop = EventLoop::new();
		let window = Window::new(&event_loop)?;

		let size = window.inner_size();
		let instance = Instance::new(Backends::all());
		let surface = unsafe { instance.create_surface(&window) };
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.await
			.context("Failed to find an appropriate adapter")?;

		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&DeviceDescriptor {
					label: None,
					features: settings.features,
					limits: Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
				},
				None,
			)
			.await
			.context("Failed to create device")?;

		let bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: None,
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: wgpu::BufferSize::new(64),
					},
					count: None,
				}],
			});

		let pipeline_layout =
			device.create_pipeline_layout(&PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[&bind_group_layout],
				push_constant_ranges: &[],
			});

		let swapchain_format = surface.get_supported_formats(&adapter)[0];

		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: None,
			source: ShaderSource::Wgsl(Cow::Borrowed(&settings.shader)),
		});

		let vertex_buffer_layout = wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[wgpu::VertexAttribute {
				format: wgpu::VertexFormat::Float32x3,
				offset: 0,
				shader_location: 0,
			}],
		};

		let render_pipeline =
			device.create_render_pipeline(&RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[vertex_buffer_layout],
				},
				fragment: Some(FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[Some(swapchain_format.into())],
				}),
				primitive: PrimitiveState::default(),
				depth_stencil: None,
				multisample: MultisampleState::default(),
				multiview: None,
			});

		let config = SurfaceConfiguration {
			usage: TextureUsages::RENDER_ATTACHMENT,
			format: swapchain_format,
			width: size.width,
			height: size.height,
			present_mode: PresentMode::Mailbox,
		};

		surface.configure(&device, &config);

		Ok(NativeApp {
			event_loop: Arc::new(RwLock::new(event_loop)),
			window: Arc::new(window),
			surface: Arc::new(surface),
			device: Arc::new(device),
			config: Arc::new(Mutex::new(config)),
			render_pipeline: Arc::new(render_pipeline),
			queue: Arc::new(queue),
			bind_group: None,
			bind_group_layout: Arc::new(bind_group_layout),
			settings,
		})
	}
}
