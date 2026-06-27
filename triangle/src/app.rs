use ash_window::enumerate_required_extensions;
use raw_window_handle::HasDisplayHandle;
use std::ffi::CStr;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

pub struct VulkanApp {
  name: String,
  window: Option<Window>,
  instance: Option<ash::Instance>,
  entry: ash::Entry,
}

impl VulkanApp {
  pub fn new(name: &str) -> Result<Self, ash::LoadingError> {
    // SAFETY: loads the system Vulkan loader; valid as long as a Vulkan ICD is installed.
    let entry = unsafe { ash::Entry::load() }?;

    Ok(Self {
      name: name.to_string(),
      window: None,
      instance: None,
      entry,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.window.is_some() && self.instance.is_some()
  }
}

impl ApplicationHandler for VulkanApp {
  // Despite the name this is called when the application is first created
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let size = LogicalSize::new(WIDTH, HEIGHT);
      let win = event_loop.create_window(Window::default_attributes().with_inner_size(size).with_title(&self.name)).unwrap();
      self.window = Some(win);
      println!("Window created with size: {:?}", size);

      self.instance = Some(unsafe {
        println!("Creating Vulkan instance...");

        // Step 1: Provide application info, not essential but recommended
        let app_name_c = std::ffi::CString::new(self.name.clone()).unwrap();
        let appinfo = ApplicationInfo::default()
          .application_name(&app_name_c)
          .application_version(0)
          .engine_name(&app_name_c)
          .engine_version(0)
          .api_version(make_api_version(0, 1, 0, 0));

        // Step 2: Enumerate required extensions for the Vulkan instance, needs ash_window & raw_window_handle
        let display_handle = event_loop.display_handle().unwrap().as_raw();
        let extensions = enumerate_required_extensions(display_handle).unwrap();
        for &ext in extensions {
          let name = CStr::from_ptr(ext);
          println!("  required extension: {}", name.to_string_lossy());
        }

        // Step 3: Add validation layers if in debug mode, to help catch mistakes in Vulkan usage
        #[cfg(debug_assertions)]
        // Needs the Vulkan SDK installed or `vulkan-validationlayers` package on Linux
        let validation_layers = [c"VK_LAYER_KHRONOS_validation"];
        #[cfg(not(debug_assertions))]
        let validation_layers = [];

        let enabled_layer_names: Vec<*const i8> = validation_layers.iter().map(|&s| s.as_ptr() as *const i8).collect();

        // Step 4: Instance creation info, is bundle of application info and extensions
        let create_info = InstanceCreateInfo::default()
          .application_info(&appinfo)
          .enabled_extension_names(extensions)
          .enabled_layer_names(&enabled_layer_names);

        // Step 5: Finally create the Vulkan instance
        self.entry.create_instance(&create_info, None).expect("Failed to create Vulkan instance")
      });

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
