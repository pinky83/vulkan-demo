use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

pub struct WindowContext {
    pub event_loop: EventLoop<()>,
    pub surface: Arc<Surface>,
    pub window: Arc<Window>,
}

impl WindowContext {
    pub fn new(instance: Arc<Instance>) -> Self {
        let event_loop = EventLoop::new();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Vulkan Triangle")
                .build(&event_loop)
                .unwrap(),
        );

        let surface =
            vulkano_win::create_surface_from_winit(window.clone(), instance.clone()).unwrap();

        Self {
            event_loop,
            surface,
            window,
        }
    }
}
