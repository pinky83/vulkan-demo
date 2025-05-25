use std::sync::Arc;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::VulkanLibrary;

pub struct VulkanDevice {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl VulkanDevice {
    pub fn new() -> Self {
        //creating Vulkan instance
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = vulkano_win::required_extensions(&library);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        // Select physical device
        let required_device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let physical = instance
            .enumerate_physical_devices()
            .unwrap()
            .find(|p| {
                p.supported_extensions()
                    .contains(&required_device_extensions)
            })
            .expect("No device found");

        // Select logical device and queue
        let (device, mut queues) = Device::new(
            physical.clone(),
            DeviceCreateInfo {
                enabled_extensions: DeviceExtensions {
                    khr_swapchain: true,
                    ..DeviceExtensions::empty()
                },
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        VulkanDevice {
            instance,
            device,
            queue,
        }
    }
}
