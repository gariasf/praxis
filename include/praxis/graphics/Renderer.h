#pragma once

#include <memory>
#include <string>
#include <vector>
#include <vulkan/vulkan.h>

struct SDL_Window;

namespace praxis::graphics
{

    /**
     * @class Renderer
     * @brief Vulkan-based rendering system
     */
    class Renderer
    {
    public:
        /**
         * @brief Constructor
         */
        Renderer();

        /**
         * @brief Destructor
         */
        ~Renderer();

        /**
         * @brief Initialize the renderer with the given SDL window
         * @param window Pointer to SDL window
         * @return True if initialization succeeded
         */
        bool initialize(SDL_Window *window);

        /**
         * @brief Begin rendering a new frame
         * @return True if begin succeeded
         */
        bool beginFrame();

        /**
         * @brief End and submit the current frame
         */
        void endFrame();

        /**
         * @brief Clean up all Vulkan resources
         */
        void cleanup();

        /**
         * @brief Check if swapchain needs recreation
         */
        void checkRecreateSwapchain();

        /**
         * @brief Handle window resize event
         * @param width New width
         * @param height New height
         */
        void handleWindowResize(int width, int height);

    private:
        /**
         * @brief Create Vulkan instance
         * @return True if creation succeeded
         */
        bool createInstance();

        /**
         * @brief Create surface for the window
         * @return True if creation succeeded
         */
        bool createSurface();

        /**
         * @brief Pick a physical device (GPU)
         * @return True if a suitable device was found
         */
        bool pickPhysicalDevice();

        /**
         * @brief Create logical device and queues
         * @return True if creation succeeded
         */
        bool createLogicalDevice();

        /**
         * @brief Create swapchain
         * @return True if creation succeeded
         */
        bool createSwapchain();

        /**
         * @brief Create image views for swapchain images
         * @return True if creation succeeded
         */
        bool createImageViews();

        /**
         * @brief Create render pass
         * @return True if creation succeeded
         */
        bool createRenderPass();

        /**
         * @brief Create framebuffers
         * @return True if creation succeeded
         */
        bool createFramebuffers();

        /**
         * @brief Create command pool
         * @return True if creation succeeded
         */
        bool createCommandPool();

        /**
         * @brief Create command buffers
         * @return True if creation succeeded
         */
        bool createCommandBuffers();

        /**
         * @brief Create synchronization objects (semaphores and fences)
         * @return True if creation succeeded
         */
        bool createSyncObjects();

        /**
         * @brief Recreate swapchain
         */
        void recreateSwapchain();

        /**
         * @brief Clean up swapchain and related resources
         */
        void cleanupSwapchain();

        /**
         * @brief Check if validation layers are supported
         * @return True if validation layers are supported
         */
        bool checkValidationLayerSupport();

        /**
         * @brief Get required instance extensions
         * @return Vector of required extension names
         */
        std::vector<const char *> getRequiredExtensions();

    private:
        SDL_Window *m_window;
        VkInstance m_instance;
        VkPhysicalDevice m_physicalDevice;
        VkDevice m_device;
        VkQueue m_graphicsQueue;
        VkQueue m_presentQueue;
        VkSurfaceKHR m_surface;
        VkSwapchainKHR m_swapchain;
        std::vector<VkImage> m_swapchainImages;
        std::vector<VkImageView> m_swapchainImageViews;
        VkFormat m_swapchainImageFormat;
        VkExtent2D m_swapchainExtent;
        VkRenderPass m_renderPass;
        std::vector<VkFramebuffer> m_swapchainFramebuffers;
        VkCommandPool m_commandPool;
        std::vector<VkCommandBuffer> m_commandBuffers;
        std::vector<VkSemaphore> m_imageAvailableSemaphores;
        std::vector<VkSemaphore> m_renderFinishedSemaphores;
        std::vector<VkFence> m_inFlightFences;
        uint32_t m_currentFrame;
        bool m_framebufferResized;

        // Debugging
        VkDebugUtilsMessengerEXT m_debugMessenger;

        static constexpr int MAX_FRAMES_IN_FLIGHT = 2;
    };

} // namespace praxis::graphics