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
  command_buffers: Option<Vec<vk::CommandBuffer>>,
  swapchain: Option<ash::vk::SwapchainKHR>,
  swapchain_loader: Option<ash::khr::swapchain::Device>,
  images: Option<Vec<vk::Image>>,
  image_views: Option<Vec<vk::ImageView>>,
  pipeline: Option<vk::Pipeline>,
  extent: Option<vk::Extent2D>,
}

impl VulkanApp {
  pub fn new(name: &str) -> Result<Self, ash::LoadingError> {
    Ok(Self {
      name: name.to_string(),
      window: None,
      instance: None,
      entry: None,
      device: None,
      command_buffers: None,
      swapchain: None,
      swapchain_loader: None,
      images: None,
      image_views: None,
      pipeline: None,
      extent: None,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.window.is_some() && self.instance.is_some() && self.entry.is_some() && self.device.is_some()
  }

  // This carries out the actual rendering commands for a given swapchain image index. It is called once per frame, and the img_index is provided by the swapchain acquire_next_image call.
  // The command buffer is recorded with the commands to transition the image layout, begin rendering, bind the pipeline, set the viewport and scissor, draw a triangle, end rendering, and transition the image layout back to present.
  fn record_command_buffer(&mut self, img_index: u32) {
    unsafe {
      let img_index = img_index as usize;
      let device = self.device.as_ref().unwrap();
      let command_buffers = self.command_buffers.as_ref().unwrap();
      let command_buffer = command_buffers[img_index];
      let extent = self.extent.unwrap();
      let image = self.images.as_ref().unwrap()[img_index];
      let image_view = self.image_views.as_ref().unwrap()[img_index];

      // Start recording commands into the command buffer
      let begin_info = vk::CommandBufferBeginInfo::default();
      device.begin_command_buffer(command_buffer, &begin_info).unwrap();

      // Insane code to say we want to transition the image layout to COLOR_ATTACHMENT_OPTIMAL
      vulkan::transition_image_layout(
        device,
        command_buffer,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::AccessFlags2::empty(),                        // src_access: nothing to wait on
        vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,         // dst_access: the upcoming colour writes
        vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, // src_stage
        vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, // dst_stage
      );

      let clear_val = vk::ClearValue {
        color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
      };

      let render_attachment = vk::RenderingAttachmentInfo::default()
        .image_view(image_view)
        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear_val);
      let attachments = [render_attachment];

      let rendering_info = vk::RenderingInfo::default()
        .render_area(vk::Rect2D {
          offset: vk::Offset2D { x: 0, y: 0 },
          extent,
        })
        .layer_count(1)
        .color_attachments(&attachments);

      device.cmd_begin_rendering(command_buffer, &rendering_info);

      // bind the graphics pipeline and draw a triangle
      device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.unwrap());

      // Set the viewport and scissor to cover the entire swapchain image
      let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.width as f32,
        height: extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
      };
      device.cmd_set_viewport(command_buffer, 0, &[viewport]);

      let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
      };
      device.cmd_set_scissor(command_buffer, 0, &[scissor]);

      device.cmd_draw(command_buffer, 3, 1, 0, 0);

      device.cmd_end_rendering(command_buffer);

      vulkan::transition_image_layout(
        device,
        command_buffer,
        image,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::ImageLayout::PRESENT_SRC_KHR,
        vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,         // src_access: the previous colour writes
        vk::AccessFlags2::empty(),                        // dst_access: nothing to wait on
        vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, // src_stage
        vk::PipelineStageFlags2::BOTTOM_OF_PIPE,          // dst_stage
      );

      device.end_command_buffer(command_buffer).unwrap();
    }
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

      // Step 1. Create Vulkan instance and get entry point
      let (entry, instance) = vulkan::init(disp_handle, &self.name, 4);

      // Step 2. Create a surface for the window and get a surface loader
      let (surface, surface_loader) = vulkan::get_surface(&entry, &instance, disp_handle, win_handle);

      // Step 3. Create a logical device and get the physical device and queue family index for graphics operations
      let (device, phys_device, qf_index) = vulkan::get_device(&instance, &surface_loader, surface);

      // Step 4. Create a swapchain for the window surface and get the swapchain loader, images, and format
      let (swapchain, _swapchain_loader, images, format, extent) = vulkan::create_swapchain(&instance, &device, phys_device, &surface_loader, surface, SIZE);

      // Step 5. Create image views for the swapchain images
      let mut image_views = vec![];
      for image in &images {
        let image_view = vulkan::create_image_view(&device, *image, format);
        image_views.push(image_view);
      }

      // Step 6. Create shader modules and stage info for vertex and fragment shaders
      let shader_mod = vulkan::create_shader_module(&device, SHADER_SPV);
      let vert_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::VERTEX, c"vertMain");
      let frag_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::FRAGMENT, c"fragMain");
      let shader_stages = vec![vert_stage, frag_stage];

      // Step 7. Create a graphics pipeline using the shader stages and swapchain/image format
      let (pipeline, _pipeline_layout) = vulkan::create_pipeline(&device, shader_stages, format);

      // Step 8. Create command pool and allocate command buffers for rendering
      let (command_buffers, _command_pool) = vulkan::allocate_command_buffers(&device, qf_index, 1);

      self.entry = Some(entry);
      self.instance = Some(instance);
      self.window = Some(win);
      self.device = Some(device);
      self.command_buffers = Some(command_buffers);
      self.swapchain = Some(swapchain);
      self.swapchain_loader = Some(_swapchain_loader);
      self.images = Some(images);
      self.image_views = Some(image_views);
      self.extent = Some(extent);
      self.pipeline = Some(pipeline);

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
        self.window.as_ref().unwrap().request_redraw();
      }

      _ => (),
    }
  }
}
