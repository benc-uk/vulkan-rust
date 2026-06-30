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
  swapchain_device: Option<ash::khr::swapchain::Device>,
  images: Option<Vec<vk::Image>>,
  image_views: Option<Vec<vk::ImageView>>,
  pipeline: Option<vk::Pipeline>,
  extent: Option<vk::Extent2D>,
  queue: vk::Queue,
  present_complete_semaphore: vk::Semaphore,
  render_complete_semaphore: vk::Semaphore,
  draw_fence: vk::Fence,
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
      swapchain_device: None,
      images: None,
      image_views: None,
      pipeline: None,
      extent: None,
      queue: vk::Queue::null(),
      present_complete_semaphore: vk::Semaphore::null(),
      render_complete_semaphore: vk::Semaphore::null(),
      draw_fence: vk::Fence::null(),
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

      // Set up the rendering attachment info for the swapchain image. This includes the image view, layout, load/store ops, and clear value.
      let render_attachment = vk::RenderingAttachmentInfo::default()
        .image_view(image_view)
        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(vk::ClearValue {
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
        });
      let attachments = [render_attachment];

      // RenderingInfo specifies the render area, layer count, and attachments.
      let rendering_info = vk::RenderingInfo::default()
        .render_area(vk::Rect2D {
          offset: vk::Offset2D { x: 0, y: 0 },
          extent,
        })
        .layer_count(1)
        .color_attachments(&attachments);

      // Start rendering with the specified rendering info. This will set up the framebuffer and begin the render pass.
      device.cmd_begin_rendering(command_buffer, &rendering_info);

      // Bind the graphics pipeline we built earlier.
      // This tells Vulkan to use our shaders and pipeline state for the upcoming draw calls.
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

      // FINALLY!! Issue the draw command to draw 3 verts, 1 instance, starting at vert 0 & instance 0
      device.cmd_draw(command_buffer, 3, 1, 0, 0);

      device.cmd_end_rendering(command_buffer);

      // Transition the image layout back to PRESENT_SRC_KHR so it can be presented to the swapchain
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

      // And stop...
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

      // Step 8. Create command pool and allocate command buffers for rendering (one per swapchain image)
      let (command_buffers, _command_pool) = vulkan::allocate_command_buffers(&device, qf_index, images.len() as u32);

      // Step 9. Store all the created Vulkan objects in the VulkanApp struct for later use
      self.entry = Some(entry);
      self.instance = Some(instance);
      self.window = Some(win);
      self.command_buffers = Some(command_buffers);
      self.swapchain = Some(swapchain);
      self.swapchain_device = Some(_swapchain_loader);
      self.images = Some(images);
      self.image_views = Some(image_views);
      self.extent = Some(extent);
      self.pipeline = Some(pipeline);
      unsafe {
        self.queue = device.get_device_queue(qf_index, 0);
        self.present_complete_semaphore = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap();
        self.render_complete_semaphore = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap();
        self.draw_fence = device.create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None).unwrap()
      }
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

      // Main loop here...
      WindowEvent::RedrawRequested => {
        let device = self.device.as_ref().unwrap();
        let draw_fence = self.draw_fence;
        let swapchain = *self.swapchain.as_ref().unwrap();
        let swapchain_device = self.swapchain_device.as_ref().unwrap();

        // This block carries out the actual rendering
        unsafe {
          // Wait draw fence to ensure the previous frame has finished rendering before we start a new one
          device.wait_for_fences(&[draw_fence], true, std::u64::MAX).unwrap();
          device.reset_fences(&[draw_fence]).unwrap();

          // Acquire the next image from the swapchain, which gives us the index of the image to render to
          // The acquire_next_image call will signal the present_complete_semaphore when the image is ready to be rendered to
          let (image_index, _) = swapchain_device
            .acquire_next_image(swapchain, std::u64::MAX, self.present_complete_semaphore, vk::Fence::null())
            .unwrap();

          self.record_command_buffer(image_index);

          // Grab semaphores and command buffers for submission to the graphics queue.
          // The wait semaphore is the present_complete_semaphore, which will be signaled when the image is ready to be rendered to.
          // The signal semaphore is the render_complete_semaphore, which will be signaled when rendering is complete and the image is ready to be presented.
          let stage_flags = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
          let command_buffers = [self.command_buffers.as_ref().unwrap()[image_index as usize]];
          let wait_semaphores = [self.present_complete_semaphore];
          let signal_semaphores = [self.render_complete_semaphore];

          // Submit the command buffer to the graphics queue for execution.
          let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&stage_flags)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

          let device = self.device.as_ref().unwrap();
          device.queue_submit(self.queue, &[submit_info], draw_fence).unwrap();

          // Present the rendered image to the swapchain.
          // Specifies the swapchain, the index of the image to present, and the semaphore to wait on before presenting
          let swapchains = [swapchain];
          let image_indices = [image_index];
          let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

          let swapchain_device = self.swapchain_device.as_ref().unwrap();
          swapchain_device.queue_present(self.queue, &present_info).unwrap();
        }

        // Redraw the window to trigger the next frame. This will cause the window_event function to be called again
        self.window.as_ref().unwrap().request_redraw();
      }

      _ => (),
    }
  }
}
