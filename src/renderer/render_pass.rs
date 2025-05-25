use std::sync::Arc;
use vulkano::device::Device;
use vulkano::render_pass::RenderPass;

pub fn create_render_pass(device: Arc<Device>, format: vulkano::format::Format) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: format,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap()
}
