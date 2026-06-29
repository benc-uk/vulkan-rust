use ash::vk;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const SHADER_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shader.spv"));

use crate::vulkan;

const SIZE: (u32, u32) = (1024, 768);

pub struct VulkanApp {
  name: String,
  window: Option<Window>,
  instance: Option<ash::Instance>,
  entry: Option<ash::Entry>,
  device: Option<ash::Device>,
}

impl VulkanApp {
  pub fn new(name: &str) -> Result<Self, ash::LoadingError> {
    Ok(Self {
      name: name.to_string(),
      window: None,
      instance: None,
      entry: None,
      device: None,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.window.is_some() && self.instance.is_some() && self.entry.is_some() && self.device.is_some()
  }
}

impl ApplicationHandler for VulkanApp {
  // Despite the name this is called when the application is first created
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let win = event_loop
        .create_window(
          Window::default_attributes()
            .with_inner_size(LogicalSize::new(SIZE.0, SIZE.1))
            .with_title(&self.name)
            .with_decorations(true),
        )
        .unwrap();

      let disp_handle = win.display_handle().unwrap().as_raw();
      let win_handle = win.window_handle().unwrap().as_raw();

      let (entry, instance) = vulkan::init(disp_handle, &self.name, 4);

      let (surface, surface_loader) = vulkan::get_surface(&entry, &instance, disp_handle, win_handle);

      let (device, phys_device, _qf_index) = vulkan::get_device(&instance, &surface_loader, surface);

      let (_swapchain, _swapchain_loader, images) = vulkan::create_swapchain(&instance, &device, phys_device, &surface_loader, surface, SIZE);

      let mut image_views = vec![];
      for image in &images {
        let image_view = vulkan::create_image_view(&device, *image);
        image_views.push(image_view);
      }

      let shader_mod = vulkan::create_shader_module(&device, SHADER_SPV);
      let vert_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::VERTEX, c"vertMain");
      let frag_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::FRAGMENT, c"fragMain");
      let _shader_stages = vec![vert_stage, frag_stage];

      vulkan::create_pipeline(&device);

      self.entry = Some(entry);
      self.instance = Some(instance);
      self.window = Some(win);
      self.device = Some(device);

      println!("App is initialized: {}", self.is_initialized());
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
