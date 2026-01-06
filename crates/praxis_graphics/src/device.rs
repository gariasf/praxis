//! Vulkan device and instance management.
//!
//! This module handles the initialization of Vulkan instances, device selection,
//! and queue creation. It provides a simplified interface for setting up the
//! Vulkan backend while handling the complexity of device enumeration and selection.

use praxis_utils::{debug, error, eyre, info, trace, Result};
use std::sync::Arc;
use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    swapchain::Surface,
    VulkanLibrary,
};
use winit::window::Window;

/// Contains the core Vulkan objects needed for rendering.
///
/// This struct holds the instance, physical device, logical device, and queues
/// that form the foundation of the Vulkan rendering system.
///
/// # Vulkan Object Hierarchy
///
/// ```text
/// VulkanLibrary
///      │
///      ▼
///   Instance ─────────────┐
///      │                  │
///      ▼                  ▼
/// PhysicalDevice      Surface (from Window)
///      │                  │
///      └──────┬───────────┘
///             ▼
///     Logical Device
///             │
///      ┌──────┴──────┐
///      ▼             ▼
/// Graphics Queue  Present Queue
/// ```
pub struct VulkanDevice {
    /// The Vulkan instance - the connection between the application and the Vulkan library.
    pub instance: Arc<Instance>,

    /// The selected physical device (GPU).
    pub physical_device: Arc<PhysicalDevice>,

    /// The logical device - our interface to the physical device.
    pub device: Arc<Device>,

    /// Queue for submitting graphics commands (rendering).
    pub graphics_queue: Arc<Queue>,

    /// Queue for presenting images to the surface (may be the same as graphics_queue).
    pub present_queue: Arc<Queue>,
}

impl VulkanDevice {
    /// Creates a new Vulkan device setup for the given window.
    ///
    /// This function:
    /// 1. Loads the Vulkan library
    /// 2. Creates a Vulkan instance with required extensions
    /// 3. Creates a surface from the window
    /// 4. Selects the best available physical device
    /// 5. Creates a logical device with graphics and presentation queues
    ///
    /// # Arguments
    ///
    /// * `window` - The window that will be used for rendering
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Vulkan library cannot be loaded
    /// - No suitable GPU is found
    /// - Device creation fails
    pub fn new(window: &Arc<Window>) -> Result<(Self, Arc<Surface>)> {
        info!("Initializing Vulkan device...");
        let device_init_start = std::time::Instant::now();

        trace!("Loading Vulkan library");
        let library = VulkanLibrary::new()
            .map_err(|e| eyre::eyre!("Failed to load Vulkan library: {}", e))?;

        let required_extensions = Surface::required_extensions(window).unwrap();
        trace!("Required instance extensions: {:?}", required_extensions);

        let enable_validation_layers = cfg!(debug_assertions);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enabled_layers: if enable_validation_layers {
                    vec![String::from("VK_LAYER_KHRONOS_validation")]
                } else {
                    vec![]
                },
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create Vulkan instance: {}", e))?;

        trace!("Created Vulkan instance");

        let layers = instance.enabled_layers();
        if layers.is_empty() {
            info!("Vulkan validation layers: disabled");
        } else {
            info!("Vulkan validation layers enabled: {:?}", layers);
        }

        trace!("Creating window surface");
        let surface = Surface::from_window(instance.clone(), window.clone())
            .map_err(|e| eyre::eyre!("Failed to create window surface: {}", e))?;

        debug!("Created window surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ext_descriptor_indexing: true,
            ..DeviceExtensions::empty()
        };

        debug!("Selecting physical device");
        let selection_start = std::time::Instant::now();
        let (physical_device, graphics_queue_family, present_queue_family) =
            Self::select_physical_device(&instance, &surface, &device_extensions)?;
        debug!(
            "Physical device selected in {:?}",
            selection_start.elapsed()
        );

        let (device, graphics_queue, present_queue) = Self::create_logical_device(
            physical_device.clone(),
            graphics_queue_family,
            present_queue_family,
            device_extensions,
        )?;

        info!(
            "Vulkan device initialization complete in {:?}",
            device_init_start.elapsed()
        );

        Ok((
            Self {
                instance,
                physical_device,
                device,
                graphics_queue,
                present_queue,
            },
            surface,
        ))
    }

    /// Selects the most suitable physical device (GPU) for rendering.
    ///
    /// The selection process:
    /// 1. Enumerates all available physical devices
    /// 2. Filters devices that support required extensions
    /// 3. Prefers discrete GPUs over integrated ones
    /// 4. Ensures the device has graphics and presentation capabilities
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The selected physical device
    /// - The queue family index for graphics operations
    /// - The queue family index for presentation operations
    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32, u32)> {
        debug!("Enumerating physical devices...");

        let devices = instance
            .enumerate_physical_devices()
            .map_err(|e| eyre::eyre!("Failed to enumerate physical devices: {}", e))?;

        info!("Found {} physical device(s)", devices.len());

        trace!("Evaluating devices for suitability");
        let suitable_device = devices
            .filter(|device| {
                let supported = device.supported_extensions().contains(device_extensions);
                if !supported {
                    trace!(
                        "Device '{}' doesn't support required extensions",
                        device.properties().device_name
                    );
                }
                supported
            })
            .filter(|device| {
                let device_type = device.properties().device_type;
                trace!(
                    "Device '{}' type: {:?}, vendor: 0x{:04x}, device: 0x{:04x}",
                    device.properties().device_name,
                    device_type,
                    device.properties().vendor_id,
                    device.properties().device_id
                );
                device_type == PhysicalDeviceType::DiscreteGpu
            })
            .filter_map(|device| {
                Self::find_queue_families(&device, surface).map(|(gfx, pres)| (device, gfx, pres))
            })
            .next()
            .ok_or_else(|| {
                error!("No suitable GPU found. Ensure a compatible Vulkan device is available.");
                eyre::eyre!("No suitable GPU found")
            })?;

        let (device, graphics_family, present_family) = suitable_device;

        info!(
            "Selected physical device: {} ({})",
            device.properties().device_name,
            match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => "Discrete GPU",
                PhysicalDeviceType::IntegratedGpu => "Integrated GPU",
                PhysicalDeviceType::VirtualGpu => "Virtual GPU",
                PhysicalDeviceType::Cpu => "CPU",
                _ => "Other",
            }
        );

        Ok((device.clone(), graphics_family, present_family))
    }

    /// Finds queue families that support graphics and presentation.
    ///
    /// Queue families are groups of queues that support specific operations.
    /// We need:
    /// - A queue family that supports graphics operations
    /// - A queue family that supports presenting to our surface
    ///
    /// These may be the same family or different families.
    ///
    /// # Returns
    ///
    /// Some((graphics_family_index, present_family_index)) if suitable families found,
    /// None otherwise.
    fn find_queue_families(device: &PhysicalDevice, surface: &Surface) -> Option<(u32, u32)> {
        let queue_families = device.queue_family_properties();

        let graphics_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, properties)| properties.queue_flags.intersects(QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32);

        let present_family = queue_families
            .iter()
            .enumerate()
            .find(|(index, _)| {
                device
                    .surface_support(*index as u32, surface)
                    .unwrap_or(false)
            })
            .map(|(index, _)| index as u32);

        match (graphics_family, present_family) {
            (Some(gfx), Some(pres)) => {
                trace!(
                    "Found queue families - Graphics: {}, Present: {}{}",
                    gfx,
                    pres,
                    if gfx == pres { " (same family)" } else { "" }
                );
                Some((gfx, pres))
            }
            _ => {
                trace!("Device doesn't support required queue families");
                None
            }
        }
    }

    /// Creates a logical device with the required queues.
    ///
    /// The logical device is our interface to the physical device. Through it,
    /// we can:
    /// - Create resources (buffers, images, etc.)
    /// - Submit commands for execution
    /// - Query device capabilities
    ///
    /// # Queue Creation
    ///
    /// If graphics and presentation use the same queue family, only one queue
    /// is created and shared. Otherwise, separate queues are created.
    fn create_logical_device(
        physical_device: Arc<PhysicalDevice>,
        graphics_queue_family: u32,
        present_queue_family: u32,
        enabled_extensions: DeviceExtensions,
    ) -> Result<(Arc<Device>, Arc<Queue>, Arc<Queue>)> {
        let mut queue_create_infos = vec![QueueCreateInfo {
            queue_family_index: graphics_queue_family,
            ..Default::default()
        }];

        if present_queue_family != graphics_queue_family {
            queue_create_infos.push(QueueCreateInfo {
                queue_family_index: present_queue_family,
                ..Default::default()
            });
        }

        debug!(
            "Creating logical device with {} queue familie(s){}",
            queue_create_infos.len(),
            if queue_create_infos.len() == 1 {
                " (unified graphics/present)"
            } else {
                ""
            }
        );

        let device_create_start = std::time::Instant::now();

        // Enable descriptor indexing features for bindless rendering
        let mut descriptor_indexing_features = DeviceFeatures::empty();
        descriptor_indexing_features.descriptor_binding_partially_bound = true;
        descriptor_indexing_features.runtime_descriptor_array = true;
        descriptor_indexing_features.descriptor_binding_variable_descriptor_count = true;
        descriptor_indexing_features.shader_sampled_image_array_non_uniform_indexing = true;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos,
                enabled_extensions,
                enabled_features: descriptor_indexing_features,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create logical device: {}", e))?;

        let graphics_queue = queues.next().unwrap();
        let present_queue = if present_queue_family != graphics_queue_family {
            queues.next().unwrap()
        } else {
            graphics_queue.clone()
        };

        debug!(
            "Created logical device and queues in {:?}",
            device_create_start.elapsed()
        );

        Ok((device, graphics_queue, present_queue))
    }
}
