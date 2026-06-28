use raw_window_handle::HasDisplayHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::vulkan;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

pub struct VulkanApp {
  name: String,
  window: Option<Window>,
  instance: Option<ash::Instance>,
  entry: Option<ash::Entry>,
}

impl VulkanApp {
  pub fn new(name: &str) -> Result<Self, ash::LoadingError> {
    Ok(Self {
      name: name.to_string(),
      window: None,
      instance: None,
      entry: None,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.window.is_some() && self.instance.is_some() && self.entry.is_some()
  }
}

impl ApplicationHandler for VulkanApp {
  // Despite the name this is called when the application is first created
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let size = LogicalSize::new(WIDTH, HEIGHT);
      let win = event_loop.create_window(Window::default_attributes().with_inner_size(size).with_title(&self.name)).unwrap();
      let display_handle = win.display_handle().unwrap().as_raw();

      let (entry, instance) = vulkan::init(display_handle, &self.name);

      let _pd = vulkan::pick_physical_device(&instance);

      self.entry = Some(entry);
      self.instance = Some(instance);
      self.window = Some(win);

      println!("Vulkan instance created. App is initialized: {}", self.is_initialized());
    }
  }

  // Trap window events, such as close requests and redraw requests
  fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }

      WindowEvent::RedrawRequested => {
        // A lot more will be going here in the future...
        self.window.as_ref().unwrap().request_redraw();
      }

      _ => (),
    }
  }
}
