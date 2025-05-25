use super::swapchain::SwapchainContext;
use std::sync::Arc;
use vulkano::image::view::ImageView;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};

pub fn create_framebuffers(
    render_pass: Arc<RenderPass>,
    swapchain_context: &SwapchainContext,
) -> Vec<Arc<Framebuffer>> {
    swapchain_context
        .images
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
        .collect()
}
