use renderer::swapchain::SwapchainContext;
use renderer::window::WindowContext;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::swapchain::{acquire_next_image, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};

mod renderer;
use renderer::device::VulkanDevice;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

fn main() {
    let vulkan_device = VulkanDevice::new();
    let window_context = WindowContext::new(vulkan_device.instance.clone());
    let swapchain_context =
        SwapchainContext::new(window_context.surface, vulkan_device.device.clone());

    let command_allocator =
        StandardCommandBufferAllocator::new(vulkan_device.device.clone(), Default::default());

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(
        vulkan_device.device.clone(),
    ));

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };
    let vertex_buffer = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        [vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "shaders/triangle.vert",
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "shaders/triangle.frag",
        }
    }

    let vs = vs::load(vulkan_device.device.clone()).unwrap();
    let fs = fs::load(vulkan_device.device.clone()).unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        vulkan_device.device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain_context.swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(vulkan_device.device.clone())
        .unwrap();

    let framebuffers: Vec<_> = swapchain_context
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
        .collect();

    let dimensions = swapchain_context.swapchain.create_info().image_extent;

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };

    window_context
        .event_loop
        .run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    _ => (),
                },
                winit::event::Event::MainEventsCleared => {
                    window_context.window.request_redraw();
                }
                winit::event::Event::RedrawRequested(_) => {
                    let (image_index, suboptimal, acquire_future) =
                        acquire_next_image(swapchain_context.swapchain.clone(), None).unwrap();
                    if suboptimal {
                        // Handle resizing later
                    }

                    let mut builder = AutoCommandBufferBuilder::primary(
                        &command_allocator,
                        vulkan_device.queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    builder
                        .begin_render_pass(
                            RenderPassBeginInfo {
                                clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                                ..RenderPassBeginInfo::framebuffer(
                                    framebuffers[image_index as usize].clone(),
                                )
                            },
                            SubpassContents::Inline,
                        )
                        .unwrap()
                        .set_viewport(0, [viewport.clone()])
                        .bind_pipeline_graphics(pipeline.clone())
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .draw(3, 1, 0, 0)
                        .unwrap()
                        .end_render_pass()
                        .unwrap();

                    let command_buffer = builder.build().unwrap();

                    let future = sync::now(vulkan_device.device.clone())
                        .join(acquire_future)
                        .then_execute(vulkan_device.queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(
                            vulkan_device.queue.clone(),
                            SwapchainPresentInfo::swapchain_image_index(
                                swapchain_context.swapchain.clone(),
                                image_index,
                            ),
                        )
                        .then_signal_fence_and_flush();

                    if let Err(e) = future {
                        eprintln!("Rendering error: {e}");
                    }
                }
                _ => (),
            }
        });
}
