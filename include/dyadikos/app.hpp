#pragma once
#define VULKAN_HPP_ENABLE_DYNAMIC_LOADER_TOOL 0
#include <SDL3/SDL.h>
#include <SDL3/SDL_vulkan.h>

#include <functional>
#include <glm/glm.hpp>
#include <memory>
#include <print>
#include <stdexcept>
#include <vulkan/vulkan_raii.hpp>

namespace dyadikos {
class SDLException final : public std::runtime_error {
 public:
  explicit SDLException(const std::string &message);
};

constexpr auto VULKAN_VERSION{vk::makeApiVersion(0, 1, 4, 0)};

struct Frame {
  vk::raii::CommandBuffer commandBuffer;
  vk::raii::Semaphore imageAvailableSemaphore;
  vk::raii::Semaphore renderFinishedSemaphore;
  vk::raii::Fence fence;
};

constexpr uint32_t IN_FLIGHT_FRAME_COUNT{2};

class App {
 public:
  App(const std::string &title = "Dyadikos");
  ~App();

  void Run(std::function<void(double)> Render);

  [[nodiscard]]
  auto BeginRender(const glm::vec4 &background_color) -> Frame const &;

  void EndRender(Frame const &frame);

 protected:
  std::unique_ptr<SDL_Window, decltype(&SDL_DestroyWindow)> window{
      nullptr, SDL_DestroyWindow};
  bool running{true};

  std::optional<vk::raii::Context> context{};
  std::optional<vk::raii::Instance> instance{};
  std::optional<vk::raii::SurfaceKHR> surface{};
  std::optional<vk::raii::PhysicalDevice> physicalDevice{};
  uint32_t graphicsQueueFamilyIndex{};
  std::optional<vk::raii::Device> device{};
  std::optional<vk::raii::Queue> graphicsQueue{};

  std::optional<vk::raii::CommandPool> commandPool{};
  std::array<std::optional<Frame>, IN_FLIGHT_FRAME_COUNT> frames{};
  uint32_t frameIndex{};

  std::optional<vk::raii::SwapchainKHR> swapchain{};
  std::vector<vk::Image> swapchainImages{};
  vk::Extent2D swapchainExtent{};
  vk::Format swapchainImageFormat{vk::Format::eB8G8R8A8Srgb};
  uint32_t currentSwapchainImageIndex{};

 private:
  struct ImageLayout {
    vk::ImageLayout imageLayout{};
    vk::PipelineStageFlags2 stageMask{};
    vk::AccessFlags2 accessMask{};
    uint32_t queueFamilyIndex{VK_QUEUE_FAMILY_IGNORED};
  };

  static void TransitionImageLayout(
      vk::raii::CommandBuffer const &commandBuffer, vk::Image const &image,
      ImageLayout const &oldLayout, ImageLayout const &newLayout);

  static void RecordCommandBuffer(vk::raii::CommandBuffer const &commandBuffer,
				  vk::Image const &swapchainImage,
				  const glm::vec4 &clearColor);

  void BeginFrame(Frame const &frame);

  void EndFrame(Frame const &frame);

  void SubmitCommandBuffer(Frame const &frame) const;

  void HandleEvents();

  void RecreateSwapchain();

  void InitFrames();

  void InitCommandPool();

  void InitDevice();

  void PickPhysicalDevice();

  void InitInstance();

  void InitSurface();
};
}  // namespace dyadikos
