use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

#[derive(Default)]
pub struct VulkanApp {
  window: Option<Window>,
  // Here you will eventually add Vulkan-related fields, such as the Vulkan instance, device, swapchain, etc.
}

impl VulkanApp {
  pub fn new() -> Self {
    Self { window: None }
  }
}

impl ApplicationHandler for VulkanApp {
  // Despite the name this is called when the application is first created
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let size = LogicalSize::new(WIDTH, HEIGHT);
      let win = event_loop
        .create_window(Window::default_attributes().with_inner_size(size).with_title("Vulkan App"))
        .unwrap();
      self.window = Some(win);
      println!("Window created with size: {:?}", size);
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }

      WindowEvent::RedrawRequested => {
        self.window.as_ref().unwrap().request_redraw();
      }

      _ => (),
    }
  }
}
