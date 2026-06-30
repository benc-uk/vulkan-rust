use ash::khr::surface;
use ash::khr::swapchain;
use ash::vk;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const SHADER_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shader.spv"));

use crate::vulkan;

const SIZE: (u32, u32) = (1024, 768);
const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanApp {
  name: String,
  window: Option<Window>,
  instance: Option<ash::Instance>,
  entry: Option<ash::Entry>,
  device: Option<ash::Device>,
  physical_device: Option<vk::PhysicalDevice>,
  command_buffers: Option<Vec<vk::CommandBuffer>>,
  swapchain: Option<ash::vk::SwapchainKHR>,
  swapchain_device: Option<ash::khr::swapchain::Device>,
  surface: Option<vk::SurfaceKHR>,
  surface_loader: Option<surface::Instance>,
  images: Option<Vec<vk::Image>>,
  image_views: Option<Vec<vk::ImageView>>,
  pipeline: Option<vk::Pipeline>,
  extent: Option<vk::Extent2D>,
  queue: vk::Queue,
  present_complete_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
  render_complete_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
  draw_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
  frame_index: usize,
}

impl VulkanApp {
  pub fn new(name: &str) -> Result<Self, ash::LoadingError> {
    Ok(Self {
      name: name.to_string(),
      window: None,
      instance: None,
      entry: None,
      device: None,
      physical_device: None,
      command_buffers: None,
      swapchain: None,
      swapchain_device: None,
      surface: None,
      surface_loader: None,
      images: None,
      image_views: None,
      pipeline: None,
      extent: None,
      queue: vk::Queue::null(),
      present_complete_semaphores: [vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT],
      render_complete_semaphores: [vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT],
      draw_fences: [vk::Fence::null(); MAX_FRAMES_IN_FLIGHT],
      frame_index: 0,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.window.is_some()
      && self.instance.is_some()
      && self.entry.is_some()
      && self.device.is_some()
      && self.physical_device.is_some()
      && self.command_buffers.is_some()
      && self.swapchain.is_some()
      && self.swapchain_device.is_some()
      && self.surface.is_some()
      && self.surface_loader.is_some()
      && self.images.is_some()
      && self.image_views.is_some()
      && self.pipeline.is_some()
      && self.extent.is_some()
  }

  // This carries out the actual rendering commands for a given swapchain image index. It is called once per frame, and the img_index is provided by the swapchain acquire_next_image call.
  // The command buffer is recorded with the commands to transition the image layout, begin rendering, bind the pipeline, set the viewport and scissor, draw a triangle, end rendering, and transition the image layout back to present.
  fn record_command_buffer(&mut self, frame_index: usize, img_index: usize) {
    unsafe {
      let device = self.device.as_ref().unwrap();
      let command_buffers = self.command_buffers.as_ref().unwrap();
      let command_buffer = command_buffers[frame_index];
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

  fn recreate_swapchain(&mut self, new_size: PhysicalSize<u32>) {
    if !self.is_initialized() {
      return;
    }

    let device = self.device.as_ref().unwrap();
    unsafe {
      device.device_wait_idle().unwrap();

      // Loop over exitsing image views and destroy them
      if let Some(image_views) = &self.image_views {
        image_views.iter().for_each(|&image_view| {
          device.destroy_image_view(image_view, None);
        });
      }

      // Destroy the old swapchain and its associated device
      if let Some(swapchain) = self.swapchain.take() {
        let swapchain_device = self.swapchain_device.take().unwrap();
        swapchain_device.destroy_swapchain(swapchain, None);
      }
    }

    let (swapchain, swapchain_device, images, format, extent) = vulkan::create_swapchain(
      self.instance.as_ref().unwrap(),
      device,
      self.physical_device.unwrap(),
      self.surface_loader.as_ref().unwrap(),
      self.surface.unwrap(),
      (new_size.width, new_size.height),
    );

    let image_views = images.iter().map(|&img| vulkan::create_image_view(device, img, format)).collect();

    self.swapchain = Some(swapchain);
    self.swapchain_device = Some(swapchain_device);
    self.images = Some(images);
    self.image_views = Some(image_views);
    self.extent = Some(extent);
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
            .with_decorations(true)
            .with_resizable(true),
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
      let (swapchain, swapchain_device, images, format, extent) = vulkan::create_swapchain(&instance, &device, phys_device, &surface_loader, surface, SIZE);

      // Step 5. Create image views for the swapchain images
      let image_views = images.iter().map(|&img| vulkan::create_image_view(&device, img, format)).collect();

      // Step 6. Create shader modules and stage info for vertex and fragment shaders
      let shader_mod = vulkan::create_shader_module(&device, SHADER_SPV);
      let vert_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::VERTEX, c"vertMain");
      let frag_stage = vulkan::create_shader_stage_info(shader_mod, vk::ShaderStageFlags::FRAGMENT, c"fragMain");
      let shader_stages = vec![vert_stage, frag_stage];

      // Step 7. Create a graphics pipeline using the shader stages and swapchain/image format
      let pipeline = vulkan::create_pipeline(&device, shader_stages, format);

      // Step 8. Create command pool and allocate command buffers for rendering (one per swapchain image)
      let (command_buffers, _command_pool) = vulkan::allocate_command_buffers(&device, qf_index, MAX_FRAMES_IN_FLIGHT as u32);

      // Step 9. Store all the created Vulkan objects in the VulkanApp struct for later use
      self.entry = Some(entry);
      self.instance = Some(instance);
      self.window = Some(win);
      self.physical_device = Some(phys_device);
      self.command_buffers = Some(command_buffers);
      self.swapchain = Some(swapchain);
      self.swapchain_device = Some(swapchain_device);
      self.surface = Some(surface);
      self.surface_loader = Some(surface_loader);
      self.images = Some(images);
      self.image_views = Some(image_views);
      self.extent = Some(extent);
      self.pipeline = Some(pipeline);
      unsafe {
        self.queue = device.get_device_queue(qf_index, 0);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
          self.present_complete_semaphores[i] = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap();
          self.render_complete_semaphores[i] = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap();
          self.draw_fences[i] = device.create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None).unwrap();
        }
      }
      self.device = Some(device);

      println!("App is initialized: {}", self.is_initialized());
    }
  }

  // Trap window events, such as close requests and redraw requests
  fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
    match event {
      WindowEvent::Resized(phys_size) => {
        // When we detect a resize - we have to recreate the swapchain and related resources
        self.recreate_swapchain(phys_size);
      }

      WindowEvent::CloseRequested => {
        event_loop.exit();
      }

      // Main loop here...
      WindowEvent::RedrawRequested => {
        let device = self.device.as_ref().unwrap();
        let draw_fence = self.draw_fences[self.frame_index];
        let swapchain = *self.swapchain.as_ref().unwrap();
        let swapchain_device = self.swapchain_device.as_ref().unwrap();

        // This block carries out the actual rendering
        unsafe {
          // Wait draw fence to ensure the previous frame has finished rendering before we start a new one
          device.wait_for_fences(&[draw_fence], true, std::u64::MAX).unwrap();

          // Acquire the next image from the swapchain, which gives us the index of the image to render to
          // The acquire_next_image call will signal the present_complete_semaphore when the image is ready to be rendered to
          let next_img_res = swapchain_device.acquire_next_image(swapchain, std::u64::MAX, self.present_complete_semaphores[self.frame_index], vk::Fence::null());

          // If the swapchain is out of date (e.g. window resized), we skip this frame,
          // The WindowEvent::Resized event will trigger a swapchain recreation, and the next frame will be rendered to the new swapchain
          if next_img_res.is_err() {
            return;
          }

          // Avoid deadlock by resetting the draw fence after aqcuiring the image, so that we can wait on it again for the next frame
          device.reset_fences(&[draw_fence]).unwrap();

          let (image_index, _is_suboptimal) = next_img_res.unwrap();

          device
            .reset_command_buffer(self.command_buffers.as_ref().unwrap()[self.frame_index], vk::CommandBufferResetFlags::empty())
            .unwrap();
          self.record_command_buffer(self.frame_index, image_index as usize);

          // Grab semaphores and command buffers for submission to the graphics queue.
          // The wait semaphore is the present_complete_semaphore, which will be signaled when the image is ready to be rendered to.
          // The signal semaphore is the render_complete_semaphore, which will be signaled when rendering is complete and the image is ready to be presented.
          let stage_flags = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
          let command_buffers = [self.command_buffers.as_ref().unwrap()[self.frame_index]];
          let wait_semaphores = [self.present_complete_semaphores[self.frame_index]];
          let signal_semaphores = [self.render_complete_semaphores[self.frame_index]];

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

        // Advance to the next frame
        self.frame_index = (self.frame_index + 1) % MAX_FRAMES_IN_FLIGHT;
      }

      _ => (),
    }
  }
}
