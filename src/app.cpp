#include <dyadikos/app.hpp>
#include <string>

namespace dyadikos {
SDLException::SDLException(const std::string &message)
    : std::runtime_error(std::format("{}: {}", message, SDL_GetError())) {}

App::App(const std::string &title) {
  if (!SDL_Init(SDL_INIT_VIDEO)) throw SDLException("Failed to initialize SDL");
  if (!SDL_Vulkan_LoadLibrary(nullptr))
    throw SDLException("Failed to load Vulkan library");
  window.reset(SDL_CreateWindow(
      "Dyadikos", 800, 600,
      SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIDDEN));
  if (!window) throw SDLException("Failed to create window");
  auto vkGetInstanceProcAddr{reinterpret_cast<PFN_vkGetInstanceProcAddr>(
      SDL_Vulkan_GetVkGetInstanceProcAddr())};
  context.emplace(vkGetInstanceProcAddr);
  auto const vulkanVersion{context->enumerateInstanceVersion()};
  std::println("Vulkan {}.{}", VK_API_VERSION_MAJOR(vulkanVersion),
	       VK_API_VERSION_MINOR(vulkanVersion));

  InitInstance();
  InitSurface();
  PickPhysicalDevice();
  InitDevice();
  InitCommandPool();
  InitFrames();
  RecreateSwapchain();
}

App::~App() {
  device->waitIdle();
  SDL_Quit();
}

void App::Run(std::function<void(double)> Render) {
  SDL_ShowWindow(window.get());
  while (running) {
    HandleEvents();
    auto const time{static_cast<double>(SDL_GetTicks())};
    Render(time);
  }
}
auto App::BeginRender(const glm::vec4 &background_color) -> Frame const & {
  auto const &frame{*frames[frameIndex]};
  BeginFrame(frame);

  RecordCommandBuffer(frame.commandBuffer,
		      swapchainImages[currentSwapchainImageIndex],
		      background_color);

  return frame;
}

void App::EndRender(Frame const &frame) {
  frame.commandBuffer.end();
  SubmitCommandBuffer(frame);
  EndFrame(frame);
}

void App::TransitionImageLayout(vk::raii::CommandBuffer const &commandBuffer,
				vk::Image const &image,
				ImageLayout const &oldLayout,
				ImageLayout const &newLayout) {
  vk::ImageMemoryBarrier2 const barrier{
      oldLayout.stageMask,
      oldLayout.accessMask,
      newLayout.stageMask,
      newLayout.accessMask,
      oldLayout.imageLayout,
      newLayout.imageLayout,
      oldLayout.queueFamilyIndex,
      newLayout.queueFamilyIndex,
      image,
      vk::ImageSubresourceRange{vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}};
  vk::DependencyInfo dependencyInfo{};
  dependencyInfo.setImageMemoryBarriers(barrier);
  commandBuffer.pipelineBarrier2(dependencyInfo);
}
void App::RecordCommandBuffer(vk::raii::CommandBuffer const &commandBuffer,
			      vk::Image const &swapchainImage,
			      const glm::vec4 &clearColor) {
  commandBuffer.reset();
  vk::CommandBufferBeginInfo beginInfo{};
  beginInfo.flags = vk::CommandBufferUsageFlagBits::eOneTimeSubmit;
  commandBuffer.begin(beginInfo);

  vk::ClearColorValue const color(clearColor.r, clearColor.g, clearColor.b,
				  clearColor.a);

  TransitionImageLayout(commandBuffer, swapchainImage,
			ImageLayout{
			    vk::ImageLayout::eUndefined,
			    vk::PipelineStageFlagBits2::eTransfer,
			    vk::AccessFlagBits2KHR::eMemoryRead,
			},
			ImageLayout{
			    vk::ImageLayout::eTransferDstOptimal,
			    vk::PipelineStageFlagBits2::eTransfer,
			    vk::AccessFlagBits2KHR::eTransferWrite,
			});

  commandBuffer.clearColorImage(
      swapchainImage, vk::ImageLayout::eTransferDstOptimal, color,
      vk::ImageSubresourceRange{vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1});

  TransitionImageLayout(commandBuffer, swapchainImage,
			ImageLayout{
			    vk::ImageLayout::eTransferDstOptimal,
			    vk::PipelineStageFlagBits2::eTransfer,
			    vk::AccessFlagBits2KHR::eTransferWrite,
			},
			ImageLayout{
			    vk::ImageLayout::ePresentSrcKHR,
			    vk::PipelineStageFlagBits2::eTransfer,
			    vk::AccessFlagBits2KHR::eMemoryRead,
			});
}
void App::BeginFrame(Frame const &frame) {
  auto _ = device->waitForFences(*frame.fence, VK_TRUE, UINT64_MAX);
  device->resetFences(*frame.fence);

  auto [acquireResult, imageIndex] = swapchain->acquireNextImage(
      UINT64_MAX, *frame.imageAvailableSemaphore, nullptr);
  currentSwapchainImageIndex = imageIndex;
}
void App::SubmitCommandBuffer(Frame const &frame) const {
  vk::SubmitInfo submitInfo{};
  submitInfo.setCommandBuffers(*frame.commandBuffer);
  submitInfo.setWaitSemaphores(*frame.imageAvailableSemaphore);
  submitInfo.setSignalSemaphores(*frame.renderFinishedSemaphore);
  constexpr vk::PipelineStageFlags waitStage{
      vk::PipelineStageFlagBits::eTransfer};
  submitInfo.setWaitDstStageMask(waitStage);
  graphicsQueue->submit(submitInfo, frame.fence);
}
void App::EndFrame(Frame const &frame) {
  vk::PresentInfoKHR presentInfo{};
  presentInfo.setSwapchains(**swapchain);
  presentInfo.setImageIndices(currentSwapchainImageIndex);
  presentInfo.setWaitSemaphores(*frame.renderFinishedSemaphore);
  auto _ = graphicsQueue->presentKHR(presentInfo);

  frameIndex = (frameIndex + 1) % IN_FLIGHT_FRAME_COUNT;
}
void App::RecreateSwapchain() {
  vk::SurfaceCapabilitiesKHR const surfaceCapabilities{
      physicalDevice->getSurfaceCapabilitiesKHR(*surface)};
  swapchainExtent = surfaceCapabilities.currentExtent;

  vk::SwapchainCreateInfoKHR swapchainCreateInfo{};
  swapchainCreateInfo.surface = *surface;
  swapchainCreateInfo.minImageCount = surfaceCapabilities.minImageCount + 1;
  swapchainCreateInfo.imageFormat = swapchainImageFormat;
  swapchainCreateInfo.imageColorSpace = vk::ColorSpaceKHR::eSrgbNonlinear;
  swapchainCreateInfo.imageExtent = swapchainExtent;
  swapchainCreateInfo.imageArrayLayers = 1;
  swapchainCreateInfo.imageUsage = vk::ImageUsageFlagBits::eColorAttachment |
				   vk::ImageUsageFlagBits::eTransferDst;
  swapchainCreateInfo.preTransform = surfaceCapabilities.currentTransform;
  swapchainCreateInfo.compositeAlpha = vk::CompositeAlphaFlagBitsKHR::eOpaque;
  swapchainCreateInfo.presentMode = vk::PresentModeKHR::eMailbox;
  swapchainCreateInfo.clipped = true;

  // todo: figure out why old swapchain handle is invalid
  // if (swapchain.has_value()) swapchainCreateInfo.oldSwapchain =
  // **swapchain;

  swapchain.emplace(*device, swapchainCreateInfo);
  swapchainImages = swapchain->getImages();
}
void App::HandleEvents() {
  for (SDL_Event event; SDL_PollEvent(&event);) switch (event.type) {
      case SDL_EVENT_QUIT:
	running = false;
	break;
      case SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED:
	RecreateSwapchain();
	break;
      default:
	break;
    }
}
void App::InitFrames() {
  vk::CommandBufferAllocateInfo commandBufferAllocateInfo{};
  commandBufferAllocateInfo.commandPool = *commandPool;
  commandBufferAllocateInfo.level = vk::CommandBufferLevel::ePrimary;
  commandBufferAllocateInfo.commandBufferCount = IN_FLIGHT_FRAME_COUNT;
  auto commandBuffers{
      device->allocateCommandBuffers(commandBufferAllocateInfo)};

  for (size_t i = 0; i < IN_FLIGHT_FRAME_COUNT; i++)
    frames[i].emplace(
	std::move(commandBuffers[i]),
	vk::raii::Semaphore{*device, vk::SemaphoreCreateInfo{}},
	vk::raii::Semaphore{*device, vk::SemaphoreCreateInfo{}},
	vk::raii::Fence{
	    *device, vk::FenceCreateInfo{vk::FenceCreateFlagBits::eSignaled}});
}
void App::InitCommandPool() {
  vk::CommandPoolCreateInfo commandPoolCreateInfo{};
  commandPoolCreateInfo.queueFamilyIndex = graphicsQueueFamilyIndex;
  commandPoolCreateInfo.flags =
      vk::CommandPoolCreateFlagBits::eResetCommandBuffer;

  commandPool.emplace(*device, commandPoolCreateInfo);
}
void App::InitDevice() {
  vk::DeviceQueueCreateInfo queueCreateInfo{};
  queueCreateInfo.queueFamilyIndex = graphicsQueueFamilyIndex;
  std::array queuePriorities{1.0f};
  queueCreateInfo.setQueuePriorities(queuePriorities);

  vk::DeviceCreateInfo deviceCreateInfo{};
  std::array queueCreateInfos{queueCreateInfo};
  deviceCreateInfo.setQueueCreateInfos(queueCreateInfos);
  std::array<const char *const, 1> enabledExtensions{
      VK_KHR_SWAPCHAIN_EXTENSION_NAME};
  deviceCreateInfo.setPEnabledExtensionNames(enabledExtensions);

  vk::PhysicalDeviceVulkan13Features vulkan13Features{};
  vulkan13Features.synchronization2 = true;

  vk::StructureChain chain{deviceCreateInfo, vulkan13Features};

  device.emplace(*physicalDevice, chain.get<vk::DeviceCreateInfo>());

  graphicsQueue.emplace(*device, graphicsQueueFamilyIndex, 0);
}
void App::PickPhysicalDevice() {
  auto const physicalDevices{instance->enumeratePhysicalDevices()};
  if (physicalDevices.empty())
    throw std::runtime_error("No Vulkan devices found");
  physicalDevice.emplace(*instance, *physicalDevices.front());
  auto const rawDeviceName{physicalDevice->getProperties().deviceName};
  std::string deviceName(rawDeviceName.data(), std::strlen(rawDeviceName));
  std::println("{}", deviceName);
  graphicsQueueFamilyIndex = 0;
}
void App::InitInstance() {
  vk::ApplicationInfo applicationInfo{};
  applicationInfo.apiVersion = VULKAN_VERSION;

  vk::InstanceCreateInfo instanceCreateInfo{};
  instanceCreateInfo.pApplicationInfo = &applicationInfo;
  uint32_t extensionCount;
  instanceCreateInfo.ppEnabledExtensionNames =
      SDL_Vulkan_GetInstanceExtensions(&extensionCount);
  instanceCreateInfo.enabledExtensionCount = extensionCount;

  instance.emplace(*context, instanceCreateInfo);
}
void App::InitSurface() {
  VkSurfaceKHR raw_surface;
  if (!SDL_Vulkan_CreateSurface(window.get(), **instance, nullptr,
				&raw_surface))
    throw SDLException("Failed to create Vulkan surface");
  surface.emplace(*instance, raw_surface);
}
}  // namespace dyadikos