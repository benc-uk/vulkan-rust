use ash::khr::{surface, swapchain};
use ash::vk;
use ash_window::{create_surface, enumerate_required_extensions};
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use std::ffi::CStr;

// NOTE! Nearly all code in here is unsafe because Vulkan is a low-level API and Rust cannot guarantee safety for it.

/// Initializes Vulkan with ash and returns the Vulkan entry and instance.
/// No error handling is performed in this function; it will panic if Vulkan initialization fails.
pub fn init(display_handle: RawDisplayHandle, app_name: &str, api_minor: u32) -> (ash::Entry, ash::Instance) {
  unsafe {
    // Step 0: Loads the system Vulkan loader; valid as long as a Vulkan ICD is installed.
    let entry = ash::Entry::load().unwrap();

    println!("Creating Vulkan instance...");

    // Step 1: Provide application info, not essential but recommended
    let app_name_c = std::ffi::CString::new(app_name).unwrap();
    let appinfo = vk::ApplicationInfo::default()
      .application_name(&app_name_c)
      .application_version(0)
      .engine_name(&app_name_c)
      .engine_version(0)
      .api_version(vk::make_api_version(0, 1, api_minor, 0));

    // Step 2: Enumerate required extensions for the Vulkan instance, needs ash_window & raw_window_handle
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
    let validation_layers: [&CStr; 0] = [];

    let enabled_layer_names: Vec<*const i8> = validation_layers.iter().map(|&s| s.as_ptr() as *const i8).collect();

    // Step 4: Instance creation info, is bundle of application info and extensions
    let create_info = vk::InstanceCreateInfo::default()
      .application_info(&appinfo)
      .enabled_extension_names(extensions)
      .enabled_layer_names(&enabled_layer_names);

    // Step 5: Finally create the Vulkan instance
    let instance = entry.create_instance(&create_info, None).expect("Failed to create Vulkan instance");

    println!("Vulkan instance created successfully.");
    (entry, instance)
  }
}

/// Picks a physical device (GPU) that will support both graphics and given surface.
/// Also ensures the logical device has swapchain extension enabled, which is required for rendering to a window surface.
/// Returns: logical  device, physical device handle, and queue family index
pub fn get_device(instance: &ash::Instance, surface_loader: &surface::Instance, surface: ash::vk::SurfaceKHR) -> (ash::Device, ash::vk::PhysicalDevice, u32) {
  unsafe {
    let physical_devices = instance.enumerate_physical_devices().expect("Failed to enumerate physical devices");
    if physical_devices.is_empty() {
      panic!("No Vulkan-compatible physical devices found");
    }

    // Define the queue flags we want to support, it's only graphics for now, but could be extended to compute, transfer, etc.
    let queue_flags = vk::QueueFlags::GRAPHICS;

    // Find a physical device that supports graphics and can present to the given surface
    let (pdevice, qf_index) = physical_devices
      .iter()
      .find_map(|pdevice| {
        instance.get_physical_device_queue_family_properties(*pdevice).iter().enumerate().find_map(|(index, info)| {
          let supports_graphic_and_surface = info.queue_flags.contains(queue_flags) && surface_loader.get_physical_device_surface_support(*pdevice, index as u32, surface).unwrap();
          if supports_graphic_and_surface { Some((*pdevice, index)) } else { None }
        })
      })
      .expect("Couldn't find suitable device.");

    // Print selected physical device properties
    let device_name = instance.get_physical_device_properties(pdevice).device_name;
    let api_version = instance.get_physical_device_properties(pdevice).api_version;
    println!("Selected physical device: {:?}", CStr::from_ptr(device_name.as_ptr()));
    println!(
      "Vulkan API version: {}.{}.{}",
      ash::vk::api_version_major(api_version),
      ash::vk::api_version_minor(api_version),
      ash::vk::api_version_patch(api_version)
    );

    let qf_index = qf_index as u32;
    let priorities = [1.0];
    let queue_info = vk::DeviceQueueCreateInfo::default().queue_family_index(qf_index).queue_priorities(&priorities);

    // Enable the swapchain extension, which is required for rendering to a window surface
    let device_extension_names_raw = [swapchain::NAME.as_ptr()];

    // Enable dynamic rendering and synchronization2 so we can use the new pipeline barrier API
    let mut vk13 = vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true).synchronization2(true);
    // Shader draw parameters is a Vulkan 1.1 feature that most Slang shaders need
    let mut vk11 = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);

    let device_create_info = vk::DeviceCreateInfo::default()
      .queue_create_infos(std::slice::from_ref(&queue_info))
      .enabled_extension_names(&device_extension_names_raw)
      .push_next(&mut vk13)
      .push_next(&mut vk11);

    let device = instance.create_device(pdevice, &device_create_info, None).unwrap();

    println!("Logical device created successfully.");

    (device, pdevice, qf_index)
  }
}

/// Obtains a Vulkan surface (SurfaceKHR) for the given window and display handles, and returns the surface and a surface loader.
pub fn get_surface(entry: &ash::Entry, instance: &ash::Instance, disp_handle: RawDisplayHandle, win_handle: RawWindowHandle) -> (ash::vk::SurfaceKHR, surface::Instance) {
  unsafe {
    let surface = create_surface(entry, instance, disp_handle, win_handle, None).expect("Failed to create Vulkan surface");
    let surface_loader = surface::Instance::new(entry, instance);

    // Just display which type of surface was created for debugging purposes
    match disp_handle {
      RawDisplayHandle::Windows(_) => println!("Created Vulkan surface for Windows."),
      RawDisplayHandle::Wayland(_) => println!("Created Vulkan surface for Wayland."),
      RawDisplayHandle::Xlib(_) => println!("Created Vulkan surface for Xlib."),
      RawDisplayHandle::Xcb(_) => println!("Created Vulkan surface for XCB."),
      RawDisplayHandle::Android(_) => println!("Created Vulkan surface for Android."),
      _ => println!("Created Vulkan surface for unknown display handle."),
    }

    (surface, surface_loader)
  }
}

/// Create a swapchain for the given surface, physical device, and logical device. Returns the swapchain and its images.
/// This function does a LOT of work, including querying surface capabilities, formats, present modes, and
/// creating the swapchain with the appropriate settings.
pub fn create_swapchain(
  instance: &ash::Instance,
  device: &ash::Device,
  phys_device: ash::vk::PhysicalDevice,
  surface_loader: &surface::Instance,
  surface: ash::vk::SurfaceKHR,
  size: (u32, u32),
) -> (ash::vk::SwapchainKHR, swapchain::Device, Vec<ash::vk::Image>, ash::vk::Format, ash::vk::Extent2D) {
  unsafe {
    // Step 1. First query the surface formats supported
    let surface_formats = surface_loader
      .get_physical_device_surface_formats(phys_device, surface)
      .expect("Failed to get surface formats");

    // Pick B8G8R8A8_SRGB if available, otherwise just pick the first format
    let surface_format = surface_formats
      .iter()
      .find(|fmt| fmt.format == ash::vk::Format::B8G8R8A8_SRGB)
      .unwrap_or(&surface_formats[0]);

    // Step 2. Query the present modes supported by the surface
    let present_modes = surface_loader
      .get_physical_device_surface_present_modes(phys_device, surface)
      .expect("Failed to get present modes");

    // Pick MAILBOX mode if available, then FIFO which we can assume is always available
    let present_mode = present_modes
      .iter()
      .find(|&&mode| mode == ash::vk::PresentModeKHR::MAILBOX)
      .unwrap_or(&ash::vk::PresentModeKHR::FIFO);

    // Step 3. Extents & image count for the swapchain, based on surface capabilities
    let surface_cap = surface_loader.get_physical_device_surface_capabilities(phys_device, surface).unwrap();
    let extent = match surface_cap.current_extent.width {
      // Max uint32 means we can choose, so we use the requested width and height
      u32::MAX => ash::vk::Extent2D { width: size.0, height: size.1 },
      _ => surface_cap.current_extent,
    };

    // Canonical way to determine the number of images
    let mut desired_img_count = surface_cap.min_image_count + 1;
    if surface_cap.max_image_count > 0 && desired_img_count > surface_cap.max_image_count {
      desired_img_count = surface_cap.max_image_count;
    }

    // Step 4. Create the swapchain
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
      .surface(surface)
      .min_image_count(desired_img_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
      .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
      .pre_transform(surface_cap.current_transform)
      .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(*present_mode)
      .clipped(true);

    let swapchain_loader = swapchain::Device::new(&instance, &device);
    let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap();
    let images = swapchain_loader.get_swapchain_images(swapchain).unwrap();

    println!(
      "Swapchain created with {} images, format: {:?}, extent: {:?}, present mode: {:?}",
      images.len(),
      surface_format.format,
      extent,
      present_mode
    );

    (swapchain, swapchain_loader, images, surface_format.format, extent)
  }
}

/// Helper function to create an image view for a given image and format. Returns the image view handle.
pub fn create_image_view(device: &ash::Device, image: ash::vk::Image, format: ash::vk::Format) -> ash::vk::ImageView {
  unsafe {
    let create_info = vk::ImageViewCreateInfo::default()
      .image(image)
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(format)
      .components(vk::ComponentMapping {
        r: vk::ComponentSwizzle::IDENTITY,
        g: vk::ComponentSwizzle::IDENTITY,
        b: vk::ComponentSwizzle::IDENTITY,
        a: vk::ComponentSwizzle::IDENTITY,
      })
      .subresource_range(vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      });

    device.create_image_view(&create_info, None).expect("Failed to create image view")
  }
}

/// Small utility function to create a shader module from SPIR-V bytecode. Returns the shader module handle.
pub fn create_shader_module(device: &ash::Device, code_bytes: &[u8]) -> ash::vk::ShaderModule {
  unsafe {
    println!("Creating shader module from SPIR-V bytecode of length: {}", code_bytes.len());

    let mut vertex_spv_file = std::io::Cursor::new(code_bytes);
    let code = ash::util::read_spv(&mut vertex_spv_file).expect("Failed to read SPIR-V code");
    let shader_info = vk::ShaderModuleCreateInfo::default().code(&code);
    device.create_shader_module(&shader_info, None).expect("Failed to create shader module")
  }
}

/// Small helper function to create a shader stage info struct for pipeline creation. Returns the shader stage info.
pub fn create_shader_stage_info<'a>(shader_module: ash::vk::ShaderModule, stage: ash::vk::ShaderStageFlags, entry_name: &'a CStr) -> ash::vk::PipelineShaderStageCreateInfo<'a> {
  println!("Creating shader stage info for stage: {:?}, entry point: {:?}", stage, entry_name);
  vk::PipelineShaderStageCreateInfo::default().module(shader_module).stage(stage).name(entry_name)
}

/// Creates a pipeline, we wil use dynamic rendering, so we don't need a render pass. Returns the pipeline handle.
/// This pipeline will be very basic, with no vertex input, no blending, no multisampling, and no depth testing. It will just render a triangle to the screen.
pub fn create_pipeline(device: &ash::Device, stages: Vec<vk::PipelineShaderStageCreateInfo>, format: vk::Format) -> (ash::vk::Pipeline, ash::vk::PipelineLayout) {
  let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
  let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

  // First set up the fixed function stages of the pipeline

  // Vertex input state, we have no vertex data for now, so it's empty
  let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

  // Input assembly state, we will use triangle list for now
  let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);

  // Viewport and scissor, we will use dynamic state for now, so we don't need to set them here, just the counts
  let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);

  // Rasterizer state, we will use fill mode and backface culling for now
  let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
    .polygon_mode(vk::PolygonMode::FILL)
    .cull_mode(vk::CullModeFlags::BACK)
    .front_face(vk::FrontFace::CLOCKWISE)
    .line_width(1.0);

  // Multisampling state, we will use no multisampling for now
  let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
    .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    .sample_shading_enable(false);

  // Colour blending state, we will use no blending for now, just write to the framebuffer
  let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::RGBA);
  let blend_state = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));

  // Now we can create the pipeline layout, which is empty for now
  let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
  let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None).expect("Failed to create pipeline layout") };

  // Dynamic rendering requires us to specify the color attachment formats in the pipeline creation info,
  // otherwise we will get a validation error. We will use the format passed as an argument, which is the format we used for the swapchain images.
  let formats = [format];
  let mut rendering_info = vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&formats);

  // Finally create the graphics pipeline, we will use dynamic rendering, so we don't need a render pass or framebuffers
  let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
    .stages(&stages)
    .vertex_input_state(&vertex_input)
    .input_assembly_state(&input_assembly)
    .viewport_state(&viewport_state)
    .rasterization_state(&rasterizer)
    .multisample_state(&multisampling)
    .color_blend_state(&blend_state)
    .layout(pipeline_layout)
    .dynamic_state(&dynamic_state)
    .push_next(&mut rendering_info);

  let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None) }.expect("Failed to create graphics pipeline")[0];

  println!("Graphics pipeline created successfully.");
  (pipeline, pipeline_layout)
}

/// Small helper function to create a command pool & buffers for the given queue family index. Returns a vector of command buffers.
pub fn allocate_command_buffers(device: &ash::Device, queue_family_index: u32, count: u32) -> (Vec<ash::vk::CommandBuffer>, ash::vk::CommandPool) {
  let pool_info = vk::CommandPoolCreateInfo::default()
    .queue_family_index(queue_family_index)
    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
  let command_pool = unsafe { device.create_command_pool(&pool_info, None).expect("Failed to create command pool") };

  let alloc_info = vk::CommandBufferAllocateInfo::default().command_pool(command_pool).command_buffer_count(count);
  let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).expect("Failed to allocate command buffers") };

  println!("Allocated {} command buffers.", command_buffers.len());
  (command_buffers, command_pool)
}

/// Records an image layout transition into the given command buffer using a synchronization2 pipeline barrier.
/// With dynamic rendering there is no render pass to do this for us, so we transition the swapchain image manually:
/// The stage/access masks describe the execution & memory dependency: src is the work that must finish first, dst is the work that must wait.
#[allow(clippy::too_many_arguments)]
pub fn transition_image_layout(
  device: &ash::Device,
  command_buffer: vk::CommandBuffer,
  image: vk::Image,
  old_layout: vk::ImageLayout,
  new_layout: vk::ImageLayout,
  src_access_mask: vk::AccessFlags2,
  dst_access_mask: vk::AccessFlags2,
  src_stage_mask: vk::PipelineStageFlags2,
  dst_stage_mask: vk::PipelineStageFlags2,
) {
  // The whole image: colour aspect, single mip level, single array layer (swapchain images are flat 2D colour images)
  let subresource_range = vk::ImageSubresourceRange::default()
    .aspect_mask(vk::ImageAspectFlags::COLOR)
    .base_mip_level(0)
    .level_count(1)
    .base_array_layer(0)
    .layer_count(1);

  // The barrier bundles the layout transition with the execution & memory dependency
  let barrier = vk::ImageMemoryBarrier2::default()
    .src_stage_mask(src_stage_mask)
    .src_access_mask(src_access_mask)
    .dst_stage_mask(dst_stage_mask)
    .dst_access_mask(dst_access_mask)
    .old_layout(old_layout)
    .new_layout(new_layout)
    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
    .image(image)
    .subresource_range(subresource_range);

  // DependencyInfo carries the array of barriers into the single pipeline_barrier2 call
  let barriers = [barrier];
  let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);

  unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency_info) };
}
