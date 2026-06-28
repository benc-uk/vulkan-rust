use ash::khr::{surface, swapchain};
use ash::vk;
use ash_window::{create_surface, enumerate_required_extensions};
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use std::ffi::CStr;

// NOTE! Nearly all code in here is unsafe because Vulkan is a low-level API and Rust cannot guarantee safety for it.

/// Initializes Vulkan with ash and returns the Vulkan entry and instance.
/// No error handling is performed in this function; it will panic if Vulkan initialization fails.
pub fn init(display_handle: RawDisplayHandle, app_name: &str) -> (ash::Entry, ash::Instance) {
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
      .api_version(vk::make_api_version(0, 1, 0, 0));

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

    let device_create_info = vk::DeviceCreateInfo::default()
      .queue_create_infos(std::slice::from_ref(&queue_info))
      .enabled_extension_names(&device_extension_names_raw);

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
) -> (ash::vk::SwapchainKHR, swapchain::Device, Vec<ash::vk::Image>) {
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

    for pmode in &present_modes {
      println!("Supported present mode: {:?}", pmode);
    }

    // Pick MAILBOX mode if available, then FIFO which we assume is always available
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

    (swapchain, swapchain_loader, images)
  }
}
