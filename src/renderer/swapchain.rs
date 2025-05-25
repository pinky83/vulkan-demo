use std::sync::Arc;
use vulkano::device::Device;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{
    CompositeAlpha, CompositeAlphas, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
};

pub struct SwapchainContext {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<SwapchainImage>>,
}

impl SwapchainContext {
    pub fn new(surface: Arc<Surface>, device: Arc<Device>) -> Self {
        let caps = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let dimensions = caps.current_extent.unwrap();

        let format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        let composite_alpha = if caps
            .supported_composite_alpha
            .contains(CompositeAlphas::OPAQUE)
        {
            CompositeAlpha::Opaque
        } else if caps
            .supported_composite_alpha
            .contains(CompositeAlphas::INHERIT)
        {
            CompositeAlpha::Inherit
        } else {
            panic!("No supported composite alpha found");
        };

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format: Some(format),
                image_extent: dimensions,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: composite_alpha,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        )
        .unwrap();

        Self { swapchain, images }
    }
}
