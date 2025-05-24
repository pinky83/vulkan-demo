use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
};
use vulkano::device::QueueFlags;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::swapchain::{
    acquire_next_image, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::swapchain::{CompositeAlpha, CompositeAlphas, PresentMode};
use vulkano::sync::{self, GpuFuture};
use vulkano::VulkanLibrary;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

fn main() {
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

    let event_loop = EventLoop::new();

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Vulkan Triangle")
            .build(&event_loop)
            .unwrap(),
    );

    let surface = vulkano_win::create_surface_from_winit(window.clone(), instance.clone()).unwrap();

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

    let caps = physical
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    let dimensions = caps.current_extent.unwrap();

    let queue_family_index = physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(index, q)| {
            q.queue_flags.contains(QueueFlags::GRAPHICS)
                && physical
                    .surface_support(index as u32, &surface)
                    .unwrap_or(false)
        })
        .expect("No compatible queue family found") as u32;

    let (device, mut queues) = Device::new(
        physical.clone(),
        DeviceCreateInfo {
            enabled_extensions: required_device_extensions.clone(),
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let queue = queues.next().unwrap();
    let caps = physical
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    let format = physical
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

    let command_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

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

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
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
    .unwrap();

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

    let framebuffers: Vec<_> = images
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

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                _ => (),
            },
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let (image_index, suboptimal, acquire_future) =
                    acquire_next_image(swapchain.clone(), None).unwrap();
                if suboptimal {
                    // Handle resizing later
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_allocator,
                    queue.queue_family_index(),
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

                let future = sync::now(device.clone())
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
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
