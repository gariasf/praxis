#include "praxis/graphics/Renderer.h"

#include "praxis/utils/Logger.h"

#include <SDL3/SDL.h>
#include <SDL3/SDL_vulkan.h>
#include <algorithm>
#include <set>

namespace praxis::graphics {

static VKAPI_ATTR VkBool32 VKAPI_CALL
debugCallback(VkDebugUtilsMessageSeverityFlagBitsEXT messageSeverity,
              VkDebugUtilsMessageTypeFlagsEXT messageType,
              const VkDebugUtilsMessengerCallbackDataEXT* pCallbackData, void* pUserData) {

  if (messageSeverity >= VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT) {
    praxis::utils::Logger::warn("Vulkan validation layer: {}", pCallbackData->pMessage);
  } else {
    praxis::utils::Logger::debug("Vulkan validation layer: {}", pCallbackData->pMessage);
  }

  // Weather we should block Vulkan operations or not
  return VK_FALSE;
}

Renderer::Renderer()
    : m_window(nullptr), m_instance(VK_NULL_HANDLE), m_physicalDevice(VK_NULL_HANDLE),
      m_device(VK_NULL_HANDLE), m_graphicsQueue(VK_NULL_HANDLE), m_presentQueue(VK_NULL_HANDLE),
      m_surface(VK_NULL_HANDLE), m_swapchain(VK_NULL_HANDLE), m_swapchainImageFormat(VK_FORMAT_UNDEFINED),
      m_swapchainExtent({0, 0}), m_renderPass(VK_NULL_HANDLE),
      m_commandPool(VK_NULL_HANDLE), m_currentFrame(0), m_framebufferResized(false),
      m_debugMessenger(VK_NULL_HANDLE) {
}

Renderer::~Renderer() { cleanup(); }

bool Renderer::initialize(SDL_Window* window) {
  m_window = window;

  utils::Logger::info("Initializing Vulkan renderer");

  if (!createInstance()) {
    utils::Logger::error("Failed to create Vulkan instance");
    return false;
  }

  // Set up the debug messenger if validation layers are enabled
#ifdef _DEBUG
  VkDebugUtilsMessengerCreateInfoEXT createInfo = {};
  createInfo.sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
  createInfo.messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT |
                               VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
                               VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
  createInfo.messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
                           VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                           VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
  createInfo.pfnUserCallback = debugCallback;
  createInfo.pUserData = nullptr;

  auto vkCreateDebugUtilsMessengerEXT = (PFN_vkCreateDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
      m_instance, "vkCreateDebugUtilsMessengerEXT");

  if (vkCreateDebugUtilsMessengerEXT) {
    if (vkCreateDebugUtilsMessengerEXT(m_instance, &createInfo, nullptr, &m_debugMessenger) !=
        VK_SUCCESS) {
      utils::Logger::warn("Failed to set up debug messenger");
    }
  } else {
    utils::Logger::warn("Failed to find vkCreateDebugUtilsMessengerEXT function");
  }
#endif

  if (!createSurface()) {
    utils::Logger::error("Failed to create window surface");
    return false;
  }

  if (!pickPhysicalDevice()) {
    utils::Logger::error("Failed to find a suitable GPU");
    return false;
  }

  if (!createLogicalDevice()) {
    utils::Logger::error("Failed to create logical device");
    return false;
  }

  if (!createSwapchain()) {
    utils::Logger::error("Failed to create swap chain");
    return false;
  }

  if (!createImageViews()) {
    utils::Logger::error("Failed to create image views");
    return false;
  }

  if (!createRenderPass()) {
    utils::Logger::error("Failed to create render pass");
    return false;
  }

  if (!createFramebuffers()) {
    utils::Logger::error("Failed to create framebuffers");
    return false;
  }

  if (!createCommandPool()) {
    utils::Logger::error("Failed to create command pool");
    return false;
  }

  if (!createCommandBuffers()) {
    utils::Logger::error("Failed to create command buffers");
    return false;
  }

  if (!createSyncObjects()) {
    utils::Logger::error("Failed to create synchronization objects");
    return false;
  }

  utils::Logger::info("Vulkan renderer initialized successfully");

  return true;
}

bool Renderer::beginFrame() {
  // Wait for the previous frame to finish
  vkWaitForFences(m_device, 1, &m_inFlightFences[m_currentFrame], VK_TRUE, UINT64_MAX);

  // Acquire the next image in the swap chain
  uint32_t imageIndex;
  VkResult result = vkAcquireNextImageKHR(m_device, m_swapchain, UINT64_MAX,
                                          m_imageAvailableSemaphores[m_currentFrame],
                                          VK_NULL_HANDLE, &imageIndex);

  if (result == VK_ERROR_OUT_OF_DATE_KHR) { 
    // Swapchain is no longer compatible with the surface
    recreateSwapchain();
    return false;
  } else if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR) {
    utils::Logger::error("Failed to acquire swap chain image");
    return false;
  }

  // Reset the fence only if we are submitting work
  vkResetFences(m_device, 1, &m_inFlightFences[m_currentFrame]);

  // Reset the command buffer to begin recording commands
  vkResetCommandBuffer(m_commandBuffers[m_currentFrame], 0);

  // Begin command buffer recording
  VkCommandBufferBeginInfo beginInfo = {};
  beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
  beginInfo.flags = 0;
  beginInfo.pInheritanceInfo = nullptr;

  if (vkBeginCommandBuffer(m_commandBuffers[m_currentFrame], &beginInfo) != VK_SUCCESS) {
    utils::Logger::error("Failed to begin recording command buffer");
    return false;
  }

  // Begin render pass
  VkRenderPassBeginInfo renderPassInfo = {};
  renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
  renderPassInfo.renderPass = m_renderPass;
  renderPassInfo.framebuffer = m_swapchainFramebuffers[imageIndex];
  renderPassInfo.renderArea.offset = {0, 0};
  renderPassInfo.renderArea.extent = m_swapchainExtent;

  VkClearValue clearColor = {{{255.0f, 255.0f, 255.0f, 1.0f}}};
  renderPassInfo.clearValueCount = 1;
  renderPassInfo.pClearValues = &clearColor;

  vkCmdBeginRenderPass(m_commandBuffers[m_currentFrame], &renderPassInfo,
                       VK_SUBPASS_CONTENTS_INLINE);

  return true;
}

void Renderer::endFrame() {
  // End render pass
  vkCmdEndRenderPass(m_commandBuffers[m_currentFrame]);

  // End command buffer recording
  if (vkEndCommandBuffer(m_commandBuffers[m_currentFrame]) != VK_SUCCESS) {
    utils::Logger::error("Failed to record command buffer");
    return;
  }

  // Submit the command buffer
  VkSubmitInfo submitInfo = {};
  submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;

  VkSemaphore waitSemaphores[] = {m_imageAvailableSemaphores[m_currentFrame]};
  VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
  submitInfo.waitSemaphoreCount = 1;
  submitInfo.pWaitSemaphores = waitSemaphores;
  submitInfo.pWaitDstStageMask = waitStages;
  submitInfo.commandBufferCount = 1;
  submitInfo.pCommandBuffers = &m_commandBuffers[m_currentFrame];

  VkSemaphore signalSemaphores[] = {m_renderFinishedSemaphores[m_currentFrame]};
  submitInfo.signalSemaphoreCount = 1;
  submitInfo.pSignalSemaphores = signalSemaphores;

  if (vkQueueSubmit(m_graphicsQueue, 1, &submitInfo, m_inFlightFences[m_currentFrame]) !=
      VK_SUCCESS) {
    utils::Logger::error("Failed to submit draw command buffer");
    return;
  }

  // Present the result back to the swap chain
  VkPresentInfoKHR presentInfo = {};
  presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
  presentInfo.waitSemaphoreCount = 1;
  presentInfo.pWaitSemaphores = signalSemaphores;

  VkSwapchainKHR swapChains[] = {m_swapchain};
  presentInfo.swapchainCount = 1;
  presentInfo.pSwapchains = swapChains;
  presentInfo.pImageIndices = &m_currentFrame;
  presentInfo.pResults = nullptr;

  VkResult result = vkQueuePresentKHR(m_presentQueue, &presentInfo);

  if (result == VK_ERROR_OUT_OF_DATE_KHR || result == VK_SUBOPTIMAL_KHR || m_framebufferResized) {
    m_framebufferResized = false;
    recreateSwapchain();
  } else if (result != VK_SUCCESS) {
    utils::Logger::error("Failed to present swap chain image");
  }

  // Advance to the next frame
  m_currentFrame = (m_currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}

void Renderer::cleanup() const {
  if (m_device == nullptr) {
    return;
  }
  // Wait for the device to finish operations before cleanup
  if (m_device != VK_NULL_HANDLE) {
    vkDeviceWaitIdle(m_device);
  }

  cleanupSwapchain();

  // Clean up synchronization objects
  for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
    if (m_renderFinishedSemaphores[i] != VK_NULL_HANDLE) {
      vkDestroySemaphore(m_device, m_renderFinishedSemaphores[i], nullptr);
    }
    if (m_imageAvailableSemaphores[i] != VK_NULL_HANDLE) {
      vkDestroySemaphore(m_device, m_imageAvailableSemaphores[i], nullptr);
    }
    if (m_inFlightFences[i] != VK_NULL_HANDLE) {
      vkDestroyFence(m_device, m_inFlightFences[i], nullptr);
    }
  }

  // Clean up the command pool
  if (m_commandPool != VK_NULL_HANDLE) {
    vkDestroyCommandPool(m_device, m_commandPool, nullptr);
  }

  // Clean up the device
  if (m_device != VK_NULL_HANDLE) {
    vkDestroyDevice(m_device, nullptr);
  }

// Clean up debug messenger
#ifdef _DEBUG
  if (m_debugMessenger != VK_NULL_HANDLE) {
    auto vkDestroyDebugUtilsMessengerEXT =
        (PFN_vkDestroyDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
            m_instance, "vkDestroyDebugUtilsMessengerEXT");
    if (vkDestroyDebugUtilsMessengerEXT) {
      vkDestroyDebugUtilsMessengerEXT(m_instance, m_debugMessenger, nullptr);
    }
  }
#endif

  // Clean up the surface
  if (m_surface != VK_NULL_HANDLE) {
    vkDestroySurfaceKHR(m_instance, m_surface, nullptr);
  }

  // Clean up the instance
  if (m_instance != VK_NULL_HANDLE) {
    vkDestroyInstance(m_instance, nullptr);
  }

  utils::Logger::info("Vulkan renderer cleaned up");
}

void Renderer::checkRecreateSwapchain() {
  if (m_framebufferResized) {
    recreateSwapchain();
    m_framebufferResized = false;
  }
}

void Renderer::handleWindowResize(int width, int height) { m_framebufferResized = true; }

bool Renderer::createInstance() {
// Check validation layer support if debug build
#ifdef _DEBUG
  if (!checkValidationLayerSupport()) {
    utils::Logger::warn("Validation layers requested, but not available");
  }
#endif

  // Application info
  VkApplicationInfo appInfo = {};
  appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
  appInfo.pApplicationName = "Praxis Engine";
  appInfo.applicationVersion = VK_MAKE_VERSION(0, 1, 0);
  appInfo.pEngineName = "Praxis";
  appInfo.engineVersion = VK_MAKE_VERSION(0, 1, 0);
  appInfo.apiVersion = VK_API_VERSION_1_4;

  // Instance create info
  VkInstanceCreateInfo createInfo = {};
  createInfo.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
  createInfo.pApplicationInfo = &appInfo;

  // Get required extensions
  auto extensions = getRequiredExtensions();
  createInfo.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
  createInfo.ppEnabledExtensionNames = extensions.data();

// Validation layers in debug mode
#ifdef _DEBUG
  const std::vector<const char*> validationLayers = {"VK_LAYER_KHRONOS_validation"};
  createInfo.enabledLayerCount = static_cast<uint32_t>(validationLayers.size());
  createInfo.ppEnabledLayerNames = validationLayers.data();

  // Debug messenger for instance creation and destruction
  VkDebugUtilsMessengerCreateInfoEXT debugCreateInfo = {};
  debugCreateInfo.sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
  debugCreateInfo.messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT |
                                    VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
                                    VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
  debugCreateInfo.messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
                                VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                                VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
  debugCreateInfo.pfnUserCallback = debugCallback;

  createInfo.pNext = &debugCreateInfo;
#else
  createInfo.enabledLayerCount = 0;
  createInfo.pNext = nullptr;
#endif

  // Create the instance
  if (vkCreateInstance(&createInfo, nullptr, &m_instance) != VK_SUCCESS) {
    utils::Logger::error("Failed to create Vulkan instance");
    return false;
  }

  utils::Logger::info("Vulkan instance created");

  // Log available extensions
  uint32_t extensionCount = 0;
  vkEnumerateInstanceExtensionProperties(nullptr, &extensionCount, nullptr);
  std::vector<VkExtensionProperties> availableExtensions(extensionCount);
  vkEnumerateInstanceExtensionProperties(nullptr, &extensionCount, availableExtensions.data());

  utils::Logger::debug("Available Vulkan extensions:");
  for (const auto& extension : availableExtensions) {
    utils::Logger::debug("  {}", extension.extensionName);
  }

  return true;
}

bool Renderer::createSurface() {
  if (SDL_Vulkan_CreateSurface(m_window, m_instance, nullptr, &m_surface) != true) {
    utils::Logger::error("Failed to create Vulkan surface: {}", SDL_GetError());
    return false;
  }

  utils::Logger::info("Vulkan surface created");
  return true;
}

bool Renderer::pickPhysicalDevice() {
  // TODO: Placeholder implementation - in a real engine, this would have a more sophisticated device
  // selection algorithm

  uint32_t deviceCount = 0;
  vkEnumeratePhysicalDevices(m_instance, &deviceCount, nullptr);

  if (deviceCount == 0) {
    utils::Logger::error("Failed to find GPUs with Vulkan support");
    return false;
  }

  std::vector<VkPhysicalDevice> devices(deviceCount);
  vkEnumeratePhysicalDevices(m_instance, &deviceCount, devices.data());

  for (const auto& device : devices) {
    VkPhysicalDeviceProperties deviceProperties;
    vkGetPhysicalDeviceProperties(device, &deviceProperties);

    VkPhysicalDeviceFeatures deviceFeatures;
    vkGetPhysicalDeviceFeatures(device, &deviceFeatures);

    utils::Logger::info("Found GPU: {}", deviceProperties.deviceName);

    // For now, just pick the first discrete GPU or any GPU if no discrete one is available
    if (deviceProperties.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) {
      m_physicalDevice = device;
      utils::Logger::info("Selected discrete GPU: {}", deviceProperties.deviceName);
      break;
    }

    // Fallback to any GPU
    if (m_physicalDevice == VK_NULL_HANDLE) {
      m_physicalDevice = device;
    }
  }

  if (m_physicalDevice == VK_NULL_HANDLE) {
    utils::Logger::error("Failed to find a suitable GPU");
    return false;
  }

  VkPhysicalDeviceProperties deviceProperties;
  vkGetPhysicalDeviceProperties(m_physicalDevice, &deviceProperties);
  utils::Logger::info("Selected GPU: {}", deviceProperties.deviceName);

  return true;
}

bool Renderer::createLogicalDevice() {
  // TODO: Simplified version - will be extended in a full engine implementation

  // Find queue families
  uint32_t queueFamilyCount = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(m_physicalDevice, &queueFamilyCount, nullptr);

  std::vector<VkQueueFamilyProperties> queueFamilies(queueFamilyCount);
  vkGetPhysicalDeviceQueueFamilyProperties(m_physicalDevice, &queueFamilyCount,
                                           queueFamilies.data());

  // Find graphics and present queue families
  uint32_t graphicsFamily = UINT32_MAX;
  uint32_t presentFamily = UINT32_MAX;

  for (uint32_t i = 0; i < queueFamilyCount; i++) {
    if (queueFamilies[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) {
      graphicsFamily = i;
    }

    VkBool32 presentSupport = false;
    vkGetPhysicalDeviceSurfaceSupportKHR(m_physicalDevice, i, m_surface, &presentSupport);

    if (presentSupport) {
      presentFamily = i;
    }

    if (graphicsFamily != UINT32_MAX && presentFamily != UINT32_MAX) {
      break;
    }
  }

  if (graphicsFamily == UINT32_MAX || presentFamily == UINT32_MAX) {
    utils::Logger::error("Failed to find suitable queue families");
    return false;
  }

  // Create a logical device with both queue families
  std::vector<VkDeviceQueueCreateInfo> queueCreateInfos;
  std::set<uint32_t> uniqueQueueFamilies = {graphicsFamily, presentFamily};

  float queuePriority = 1.0f;
  for (uint32_t queueFamily : uniqueQueueFamilies) {
    VkDeviceQueueCreateInfo queueCreateInfo = {};
    queueCreateInfo.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queueCreateInfo.queueFamilyIndex = queueFamily;
    queueCreateInfo.queueCount = 1;
    queueCreateInfo.pQueuePriorities = &queuePriority;
    queueCreateInfos.push_back(queueCreateInfo);
  }

  // Device features
  VkPhysicalDeviceFeatures deviceFeatures = {};
  // Enable features as needed

  // Device extensions
  std::vector<const char*> deviceExtensions = {VK_KHR_SWAPCHAIN_EXTENSION_NAME};

  // Device create info
  VkDeviceCreateInfo createInfo = {};
  createInfo.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
  createInfo.queueCreateInfoCount = static_cast<uint32_t>(queueCreateInfos.size());
  createInfo.pQueueCreateInfos = queueCreateInfos.data();
  createInfo.pEnabledFeatures = &deviceFeatures;
  createInfo.enabledExtensionCount = static_cast<uint32_t>(deviceExtensions.size());
  createInfo.ppEnabledExtensionNames = deviceExtensions.data();

// Validation layers in debug mode (for backward compatibility)
#ifdef _DEBUG
  const std::vector<const char*> validationLayers = {"VK_LAYER_KHRONOS_validation"};
  createInfo.enabledLayerCount = static_cast<uint32_t>(validationLayers.size());
  createInfo.ppEnabledLayerNames = validationLayers.data();
#else
  createInfo.enabledLayerCount = 0;
#endif

  // Create the logical device
  if (vkCreateDevice(m_physicalDevice, &createInfo, nullptr, &m_device) != VK_SUCCESS) {
    utils::Logger::error("Failed to create logical device");
    return false;
  }

  // Get queue handles
  vkGetDeviceQueue(m_device, graphicsFamily, 0, &m_graphicsQueue);
  vkGetDeviceQueue(m_device, presentFamily, 0, &m_presentQueue);

  utils::Logger::info("Logical device created");
  return true;
}

bool Renderer::createSwapchain() {
  // TODO: For now, this is just a very simplified version
  // In a real engine, you would want to query for capabilities, formats, etc.

  int width, height;
  SDL_GetWindowSizeInPixels(m_window, &width, &height);

  m_swapchainExtent = {static_cast<uint32_t>(width), static_cast<uint32_t>(height)};
  m_swapchainImageFormat = VK_FORMAT_B8G8R8A8_SRGB; // Typically a good default

  // Create a basic swapchain
  VkSwapchainCreateInfoKHR createInfo = {};
  createInfo.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
  createInfo.surface = m_surface;
  createInfo.minImageCount = 2; // Double buffering
  createInfo.imageFormat = m_swapchainImageFormat;
  createInfo.imageColorSpace = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR;
  createInfo.imageExtent = m_swapchainExtent;
  createInfo.imageArrayLayers = 1;
  createInfo.imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
  createInfo.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;
  createInfo.preTransform = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
  createInfo.compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
  createInfo.presentMode = VK_PRESENT_MODE_FIFO_KHR; // Guaranteed to be available
  createInfo.clipped = VK_TRUE;
  createInfo.oldSwapchain = VK_NULL_HANDLE;

  if (vkCreateSwapchainKHR(m_device, &createInfo, nullptr, &m_swapchain) != VK_SUCCESS) {
    utils::Logger::error("Failed to create swap chain");
    return false;
  }

  // Get the swapchain images
  uint32_t imageCount;
  vkGetSwapchainImagesKHR(m_device, m_swapchain, &imageCount, nullptr);
  m_swapchainImages.resize(imageCount);
  vkGetSwapchainImagesKHR(m_device, m_swapchain, &imageCount, m_swapchainImages.data());

  utils::Logger::info("Swapchain created with {} images at resolution {}x{}", imageCount,
                      m_swapchainExtent.width, m_swapchainExtent.height);

  return true;
}

bool Renderer::createImageViews() {
  m_swapchainImageViews.resize(m_swapchainImages.size());

  for (size_t i = 0; i < m_swapchainImages.size(); i++) {
    VkImageViewCreateInfo createInfo = {};
    createInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    createInfo.image = m_swapchainImages[i];
    createInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    createInfo.format = m_swapchainImageFormat;
    createInfo.components.r = VK_COMPONENT_SWIZZLE_IDENTITY;
    createInfo.components.g = VK_COMPONENT_SWIZZLE_IDENTITY;
    createInfo.components.b = VK_COMPONENT_SWIZZLE_IDENTITY;
    createInfo.components.a = VK_COMPONENT_SWIZZLE_IDENTITY;
    createInfo.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    createInfo.subresourceRange.baseMipLevel = 0;
    createInfo.subresourceRange.levelCount = 1;
    createInfo.subresourceRange.baseArrayLayer = 0;
    createInfo.subresourceRange.layerCount = 1;

    if (vkCreateImageView(m_device, &createInfo, nullptr, &m_swapchainImageViews[i]) !=
        VK_SUCCESS) {
      utils::Logger::error("Failed to create image views");
      return false;
    }
  }

  utils::Logger::info("Image views created");
  return true;
}

bool Renderer::createRenderPass() {
  // Color attachment description
  VkAttachmentDescription colorAttachment = {};
  colorAttachment.format = m_swapchainImageFormat;
  colorAttachment.samples = VK_SAMPLE_COUNT_1_BIT;
  colorAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
  colorAttachment.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
  colorAttachment.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
  colorAttachment.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
  colorAttachment.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
  colorAttachment.finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;

  // Subpass description
  VkAttachmentReference colorAttachmentRef = {};
  colorAttachmentRef.attachment = 0;
  colorAttachmentRef.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;

  VkSubpassDescription subpass = {};
  subpass.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
  subpass.colorAttachmentCount = 1;
  subpass.pColorAttachments = &colorAttachmentRef;

  // Dependency for synchronization
  VkSubpassDependency dependency = {};
  dependency.srcSubpass = VK_SUBPASS_EXTERNAL;
  dependency.dstSubpass = 0;
  dependency.srcStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
  dependency.srcAccessMask = 0;
  dependency.dstStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
  dependency.dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;

  // Render pass create info
  VkRenderPassCreateInfo renderPassInfo = {};
  renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
  renderPassInfo.attachmentCount = 1;
  renderPassInfo.pAttachments = &colorAttachment;
  renderPassInfo.subpassCount = 1;
  renderPassInfo.pSubpasses = &subpass;
  renderPassInfo.dependencyCount = 1;
  renderPassInfo.pDependencies = &dependency;

  if (vkCreateRenderPass(m_device, &renderPassInfo, nullptr, &m_renderPass) != VK_SUCCESS) {
    utils::Logger::error("Failed to create render pass");
    return false;
  }

  utils::Logger::info("Render pass created");
  return true;
}

bool Renderer::createFramebuffers() {
  m_swapchainFramebuffers.resize(m_swapchainImageViews.size());

  for (size_t i = 0; i < m_swapchainImageViews.size(); i++) {
    VkImageView attachments[] = {m_swapchainImageViews[i]};

    VkFramebufferCreateInfo framebufferInfo = {};
    framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
    framebufferInfo.renderPass = m_renderPass;
    framebufferInfo.attachmentCount = 1;
    framebufferInfo.pAttachments = attachments;
    framebufferInfo.width = m_swapchainExtent.width;
    framebufferInfo.height = m_swapchainExtent.height;
    framebufferInfo.layers = 1;

    if (vkCreateFramebuffer(m_device, &framebufferInfo, nullptr, &m_swapchainFramebuffers[i]) !=
        VK_SUCCESS) {
      utils::Logger::error("Failed to create framebuffer");
      return false;
    }
  }

  utils::Logger::info("Framebuffers created");
  return true;
}

bool Renderer::createCommandPool() {
  // Find queue family index with graphics support
  uint32_t queueFamilyIndex = 0;
  uint32_t queueFamilyCount = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(m_physicalDevice, &queueFamilyCount, nullptr);

  std::vector<VkQueueFamilyProperties> queueFamilies(queueFamilyCount);
  vkGetPhysicalDeviceQueueFamilyProperties(m_physicalDevice, &queueFamilyCount,
                                           queueFamilies.data());

  for (uint32_t i = 0; i < queueFamilyCount; i++) {
    if (queueFamilies[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) {
      queueFamilyIndex = i;
      break;
    }
  }

  VkCommandPoolCreateInfo poolInfo = {};
  poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
  poolInfo.queueFamilyIndex = queueFamilyIndex;
  poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;

  if (vkCreateCommandPool(m_device, &poolInfo, nullptr, &m_commandPool) != VK_SUCCESS) {
    utils::Logger::error("Failed to create command pool");
    return false;
  }

  utils::Logger::info("Command pool created");
  return true;
}

bool Renderer::createCommandBuffers() {
  m_commandBuffers.resize(MAX_FRAMES_IN_FLIGHT);

  VkCommandBufferAllocateInfo allocInfo = {};
  allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
  allocInfo.commandPool = m_commandPool;
  allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
  allocInfo.commandBufferCount = static_cast<uint32_t>(m_commandBuffers.size());

  if (vkAllocateCommandBuffers(m_device, &allocInfo, m_commandBuffers.data()) != VK_SUCCESS) {
    utils::Logger::error("Failed to allocate command buffers");
    return false;
  }

  utils::Logger::info("Command buffers created");
  return true;
}

bool Renderer::createSyncObjects() {
  m_imageAvailableSemaphores.resize(MAX_FRAMES_IN_FLIGHT);
  m_renderFinishedSemaphores.resize(MAX_FRAMES_IN_FLIGHT);
  m_inFlightFences.resize(MAX_FRAMES_IN_FLIGHT);

  VkSemaphoreCreateInfo semaphoreInfo = {};
  semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;

  VkFenceCreateInfo fenceInfo = {};
  fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
  fenceInfo.flags = VK_FENCE_CREATE_SIGNALED_BIT; // Create in signaled state so we don't wait
                                                  // indefinitely on the first frame

  for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
    if (vkCreateSemaphore(m_device, &semaphoreInfo, nullptr, &m_imageAvailableSemaphores[i]) !=
            VK_SUCCESS ||
        vkCreateSemaphore(m_device, &semaphoreInfo, nullptr, &m_renderFinishedSemaphores[i]) !=
            VK_SUCCESS ||
        vkCreateFence(m_device, &fenceInfo, nullptr, &m_inFlightFences[i]) != VK_SUCCESS) {

      utils::Logger::error("Failed to create synchronization objects");
      return false;
    }
  }

  utils::Logger::info("Synchronization objects created");
  return true;
}

void Renderer::recreateSwapchain() {
  // Wait for the device to finish
  vkDeviceWaitIdle(m_device);

  // Clean up old swapchain resources
  cleanupSwapchain();

  // Recreate swapchain and resources
  createSwapchain();
  createImageViews();
  createRenderPass();
  createFramebuffers();

  utils::Logger::info("Swapchain recreated");
}

void Renderer::cleanupSwapchain() const {
  // Make sure the device is valid before cleaning up
  if (m_device == VK_NULL_HANDLE) {
    return;
  }

  // Clean up framebuffers
  for (const auto framebuffer : m_swapchainFramebuffers) {
    if (framebuffer != VK_NULL_HANDLE) {
      vkDestroyFramebuffer(m_device, framebuffer, nullptr);
    }
  }

  // Clean up render pass
  if (m_renderPass != VK_NULL_HANDLE) {
    vkDestroyRenderPass(m_device, m_renderPass, nullptr);
  }

  // Clean up image views
  for (auto imageView : m_swapchainImageViews) {
    if (imageView != VK_NULL_HANDLE) {
      vkDestroyImageView(m_device, imageView, nullptr);
    }
  }

  // Clean up swapchain
  if (m_swapchain != VK_NULL_HANDLE) {
    vkDestroySwapchainKHR(m_device, m_swapchain, nullptr);
  }
}

bool Renderer::checkValidationLayerSupport() {
  uint32_t layerCount;
  vkEnumerateInstanceLayerProperties(&layerCount, nullptr);

  std::vector<VkLayerProperties> availableLayers(layerCount);
  vkEnumerateInstanceLayerProperties(&layerCount, availableLayers.data());

  for (const auto& layerProperties : availableLayers) {
    utils::Logger::debug("Available layer: {}", layerProperties.layerName);
  }

  const std::vector<const char*> validationLayers = {"VK_LAYER_KHRONOS_validation"};

  for (const char* layerName : validationLayers) {
    bool layerFound = false;

    for (const auto& layerProperties : availableLayers) {
      if (strcmp(layerName, layerProperties.layerName) == 0) {
        layerFound = true;
        break;
      }
    }

    if (!layerFound) {
      return false;
    }
  }

  return true;
}

std::vector<const char*> Renderer::getRequiredExtensions() {
  // Get SDL required extensions
  uint32_t extensionCount = 0;
  const char* const* extensionNames = SDL_Vulkan_GetInstanceExtensions(&extensionCount);

  if (!extensionNames) {
    utils::Logger::error("Failed to get Vulkan extensions: {}", SDL_GetError());
    return {};
  }

  std::vector<const char*> extensions(extensionNames, extensionNames + extensionCount);

// Add debug extension in debug builds
#ifdef _DEBUG
  extensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
#endif

  return extensions;
}

} // namespace praxis::graphics