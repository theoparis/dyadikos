use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, Mutex};
use tracing::debug;
use vulkano::{
	command_buffer::{
		AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
	},
	device::{
		physical::PhysicalDevice, Device, DeviceExtensions, Queue,
		QueueCreateInfo,
	},
	image::{view::ImageView, ImageAccess, ImageUsage, SwapchainImage},
	impl_vertex,
	instance::Instance,
	pipeline::{
		graphics::{
			input_assembly::InputAssemblyState,
			vertex_input::BuffersDefinition,
			viewport::{Viewport, ViewportState},
		},
		GraphicsPipeline,
	},
	render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
	shader::ShaderModule,
	swapchain::{
		acquire_next_image, AcquireError, Surface, Swapchain,
		SwapchainCreateInfo, SwapchainCreationError,
	},
	sync::{FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::run_return::EventLoopExtRunReturn,
	window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
	pub position: [f32; 3],
}
impl_vertex!(Vertex, position);

#[derive(Clone)]
pub struct VulkanApp {
	pub swapchain: Arc<Swapchain<Window>>,
	pub queue: Arc<Queue>,
	pub device: Arc<Device>,
	pub surface: Arc<Surface<Window>>,
	pub images: Vec<Arc<SwapchainImage<Window>>>,
	pub event_loop: Arc<Mutex<EventLoop<()>>>,
}

impl VulkanApp {
	pub fn new(device_id: usize) -> Result<Self> {
		let required_extensions = vulkano_win::required_extensions();
		let instance = Instance::new(vulkano::instance::InstanceCreateInfo {
			enabled_extensions: required_extensions,
			..Default::default()
		})?;

		let physical_device = PhysicalDevice::enumerate(&instance)
			.nth(device_id)
			.context("Failed to find a suitable graphics device")?;

		debug!(
			"Starting vulkan app with physical device: {}",
			physical_device.properties().device_name
		);

		let event_loop = EventLoop::new();
		let surface = WindowBuilder::new()
			.build_vk_surface(&event_loop, instance.clone())
			.context("Failed to create a window")?;

		let queue_family = physical_device
			.queue_families()
			.find(|&q| {
				q.supports_graphics()
					&& q.supports_surface(&surface).unwrap_or(false)
			})
			.context("Failed to find a supported queue family")?;

		let device_extensions = DeviceExtensions {
			khr_swapchain: true,
			..DeviceExtensions::none()
		};

		let (device, mut queues) = Device::new(
			physical_device,
			vulkano::device::DeviceCreateInfo {
				enabled_extensions: physical_device
					.required_extensions()
					.union(&device_extensions),
				queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
				..Default::default()
			},
		)
		.context("Failed to create vulkan device")?;

		let queue = queues.next().unwrap();

		let (swapchain, images) = {
			let surface_capabilities = physical_device
				.surface_capabilities(&surface, Default::default())?;

			let image_format = Some(
				physical_device
					.surface_formats(&surface, Default::default())?[0]
					.0,
			);

			Swapchain::new(
				device.clone(),
				surface.clone(),
				SwapchainCreateInfo {
					min_image_count: surface_capabilities.min_image_count,
					image_format,
					image_extent: surface.window().inner_size().into(),
					image_usage: ImageUsage::color_attachment(),
					composite_alpha: surface_capabilities
						.supported_composite_alpha
						.iter()
						.next()
						.unwrap(),
					..Default::default()
				},
			)
			.context("Failed to create swapchain")?
		};

		Ok(VulkanApp {
			queue,
			swapchain,
			device,
			images,
			event_loop: Arc::new(Mutex::new(event_loop)),
			surface,
		})
	}

	#[allow(clippy::type_complexity)]
	pub fn run(
		&mut self,
		vertex_shader: Arc<ShaderModule>,
		fragment_shader: Arc<ShaderModule>,
		mut callback: Box<
			dyn FnMut(
				&mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
				Arc<Framebuffer>,
				Viewport,
				Arc<GraphicsPipeline>,
			),
		>,
	) -> Result<()> {
		let render_pass = vulkano::single_pass_renderpass!(
			self.device.clone(),
			attachments: {
				color: {
					load: Clear,
					store: Store,
					format: self.swapchain.image_format(),
					samples: 1,
				}
			},
			pass: {
				color: [color],
				depth_stencil: {}
			}
		)
		.context("Failed to create render pass")?;

		let pipeline = GraphicsPipeline::start()
			.vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
			.vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
			.input_assembly_state(InputAssemblyState::new())
			.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
			.fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(self.device.clone())
			.context("Failed to create graphics pipeline")
			.unwrap();

		let mut viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [0.0, 0.0],
			depth_range: 0.0..1.0,
		};

		let mut framebuffers = VulkanApp::window_size_dependent_setup(
			&self.images,
			render_pass.clone(),
			&mut viewport,
		);

		let mut recreate_swapchain = false;
		let mut previous_frame_end =
			Some(vulkano::sync::now(self.device.clone()).boxed());

		self.event_loop
			.clone()
			.as_ref()
			.try_lock()
			.unwrap()
			.run_return(move |event, _, control_flow| match event {
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => {
					*control_flow = ControlFlow::Exit;
				}
				Event::WindowEvent {
					event: WindowEvent::Resized(_),
					..
				} => {
					recreate_swapchain = true;
				}
				Event::RedrawEventsCleared => {
					previous_frame_end.as_mut().unwrap().cleanup_finished();

					if recreate_swapchain {
						let (new_swapchain, new_images) = match self
						.swapchain
						.recreate(SwapchainCreateInfo {
							image_extent: self
								.surface
								.window()
								.inner_size()
								.into(),
							..self.swapchain.create_info()
						}) {
						Ok(r) => r,
						Err(
							SwapchainCreationError::ImageExtentNotSupported {
								..
							},
						) => return,
						Err(e) => {
							panic!("Failed to recreate swapchain: {:?}", e)
						}
					};

						self.swapchain = new_swapchain;
						framebuffers = VulkanApp::window_size_dependent_setup(
							&new_images,
							render_pass.clone(),
							&mut viewport,
						);
						recreate_swapchain = false;
					}

					let (image_num, suboptimal, acquire_future) =
						match acquire_next_image(self.swapchain.clone(), None) {
							Ok(r) => r,
							Err(AcquireError::OutOfDate) => {
								recreate_swapchain = true;
								return;
							}
							Err(e) => {
								panic!("Failed to acquire next image: {:?}", e)
							}
						};

					if suboptimal {
						recreate_swapchain = true;
					}

					let mut builder = AutoCommandBufferBuilder::primary(
						self.device.clone(),
						self.queue.family(),
						CommandBufferUsage::OneTimeSubmit,
					)
					.unwrap();

					callback(
						&mut builder,
						framebuffers[image_num].clone(),
						viewport.clone(),
						pipeline.clone(),
					);

					let command_buffer = builder.build().unwrap();

					let future = previous_frame_end
						.take()
						.unwrap()
						.join(acquire_future)
						.then_execute(self.queue.clone(), command_buffer)
						.unwrap()
						.then_swapchain_present(
							self.queue.clone(),
							self.swapchain.clone(),
							image_num,
						)
						.then_signal_fence_and_flush();

					match future {
						Ok(future) => {
							previous_frame_end = Some(future.boxed());
						}
						Err(FlushError::OutOfDate) => {
							recreate_swapchain = true;
							previous_frame_end = Some(
								vulkano::sync::now(self.device.clone()).boxed(),
							);
						}
						Err(e) => {
							println!("Failed to flush future: {:?}", e);
							previous_frame_end = Some(
								vulkano::sync::now(self.device.clone()).boxed(),
							);
						}
					}
				}
				_ => (),
			});

		Ok(())
	}

	fn window_size_dependent_setup(
		images: &[Arc<SwapchainImage<Window>>],
		render_pass: Arc<RenderPass>,
		viewport: &mut Viewport,
	) -> Vec<Arc<Framebuffer>> {
		let dimensions = images[0].dimensions().width_height();
		viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

		images
			.iter()
			.map(|image| {
				let view = ImageView::new_default(image.clone()).unwrap();
				Framebuffer::new(
					render_pass.clone(),
					FramebufferCreateInfo {
						attachments: vec![view],
						..Default::default()
					},
				)
				.unwrap()
			})
			.collect::<Vec<_>>()
	}
}

pub mod mesh;
pub mod shader;
