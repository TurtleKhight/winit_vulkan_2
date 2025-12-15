use std::sync::Arc;

use raw_window_handle::HasDisplayHandle;
use vulkano::{
    VulkanLibrary,
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    swapchain::Surface,
};
mod config;

pub use config::VulkanConfig;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub cmd_alloc: Arc<StandardCommandBufferAllocator>,
    pub mem_alloc: Arc<StandardMemoryAllocator>,
    pub set_alloc: Arc<StandardDescriptorSetAllocator>,

    pub config: VulkanConfig,
}
impl VulkanContext {
    pub fn new(display: &impl HasDisplayHandle, config: VulkanConfig) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_instance_extensions = Surface::required_extensions(&display).unwrap();
        let instance_create_info = InstanceCreateInfo {
            enabled_extensions: required_instance_extensions,
            ..Default::default()
        };
        let instance = Instance::new(library, instance_create_info).unwrap();

        Self::from_instance(display, instance, config)
    }

    pub fn from_instance(
        display: &impl HasDisplayHandle,
        instance: Arc<Instance>,
        mut config: VulkanConfig,
    ) -> Self {
        let required_device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let required_queue_flags =
            QueueFlags::GRAPHICS | QueueFlags::COMPUTE | QueueFlags::TRANSFER;

        let device_candidates = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| {
                p.supported_extensions()
                    .contains(&required_device_extensions)
            })
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(required_queue_flags)
                            && p.presentation_support(i as u32, display).unwrap()
                    })
                    .map(|i| (p, i as u32))
            });

        if config.needs_devices() {
            config.clear_devices();
            for physical_device in instance.enumerate_physical_devices().unwrap() {
                config.add_device(physical_device.clone());
            }
        }
        let (physical_device, queue_family_index) =
            if let Some(target_device) = config.target_device() {
                find_device(device_candidates, target_device).unwrap()
            } else {
                pick_device_match(device_candidates).unwrap()
            };
        config.set_current_device(physical_device.clone());

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: required_device_extensions,

                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],

                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let mem_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let cmd_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let set_alloc = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            instance,
            device,
            queue,
            cmd_alloc,
            mem_alloc,
            set_alloc,
            config,
        }
    }
}

fn pick_device_match(
    device_candidates: impl Iterator<Item = (Arc<PhysicalDevice>, u32)>,
) -> Option<(Arc<PhysicalDevice>, u32)> {
    device_candidates.min_by_key(|(p, _)| match p.properties().device_type {
        PhysicalDeviceType::DiscreteGpu => 0,
        PhysicalDeviceType::IntegratedGpu => 1,
        PhysicalDeviceType::VirtualGpu => 2,
        PhysicalDeviceType::Cpu => 3,
        PhysicalDeviceType::Other => 4,
        _ => 5,
    })
}

fn find_device(
    device_candidates: impl Iterator<Item = (Arc<PhysicalDevice>, u32)>,
    device_id: u32,
) -> Option<(Arc<PhysicalDevice>, u32)> {
    for candidate in device_candidates {
        if candidate.0.properties().device_id == device_id {
            return Some(candidate);
        }
    }
    None
    // device_candidates.find(|(pd, _)| pd.properties().device_id == device_id)
}
