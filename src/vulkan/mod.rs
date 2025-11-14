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

use crate::{msg, msgln};

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub cmd_alloc: Arc<StandardCommandBufferAllocator>,
    pub mem_alloc: Arc<StandardMemoryAllocator>,
    pub set_alloc: Arc<StandardDescriptorSetAllocator>,
}
impl VulkanContext {
    pub fn new(display: &impl HasDisplayHandle) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_instance_extensions = Surface::required_extensions(&display).unwrap();
        let instance_create_info = InstanceCreateInfo {
            enabled_extensions: required_instance_extensions,
            ..Default::default()
        };
        let instance = Instance::new(library, instance_create_info).unwrap();

        msgln!("All Devices: ");
        for physical_device in instance.enumerate_physical_devices().unwrap() {
            print!("   - ");
            print_infos(&physical_device);
        }

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

        let (physical_device, queue_family_index) = device_candidates
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

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
        }
    }
}

fn print_infos(dev: &PhysicalDevice) {
    println!(
        "{} ({:?})",
        dev.properties().device_name,
        dev.properties().device_type
    );
}
