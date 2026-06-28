use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version};
use ash_window::enumerate_required_extensions;
use raw_window_handle::RawDisplayHandle;
use std::ffi::CStr;

/// Initializes Vulkan with ash and returns the Vulkan entry and instance.
/// This function is unsafe because it involves raw pointers and FFI calls
/// No error handling is performed in this function; it will panic if Vulkan initialization fails.
pub fn init(display_handle: RawDisplayHandle, app_name: &str) -> (ash::Entry, ash::Instance) {
  unsafe {
    // Step 0: Loads the system Vulkan loader; valid as long as a Vulkan ICD is installed.
    let entry = ash::Entry::load().unwrap();

    println!("Creating Vulkan instance...");

    // Step 1: Provide application info, not essential but recommended
    let app_name_c = std::ffi::CString::new(app_name).unwrap();
    let appinfo = ApplicationInfo::default()
      .application_name(&app_name_c)
      .application_version(0)
      .engine_name(&app_name_c)
      .engine_version(0)
      .api_version(make_api_version(0, 1, 0, 0));

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
    let create_info = InstanceCreateInfo::default()
      .application_info(&appinfo)
      .enabled_extension_names(extensions)
      .enabled_layer_names(&enabled_layer_names);

    // Step 5: Finally create the Vulkan instance
    let instance = entry.create_instance(&create_info, None).expect("Failed to create Vulkan instance");

    println!("Vulkan instance created successfully.");
    (entry, instance)
  }
}

/// Picks a physical device (GPU) from the available devices on the system.
/// This function enumerates all physical devices and prints their properties,
/// No smart selection is performed; it simply returns the device at the given index.
pub fn get_physical_device(instance: &ash::Instance, index: usize) -> ash::vk::PhysicalDevice {
  unsafe {
    let physical_devices = instance.enumerate_physical_devices().expect("Failed to enumerate physical devices");
    if physical_devices.is_empty() {
      panic!("No Vulkan-compatible physical devices found");
    }

    // Enumerate and print information about each physical device, for diagnostic purposes only
    for (i, device) in physical_devices.iter().enumerate() {
      let properties = instance.get_physical_device_properties(*device);
      let device_name = CStr::from_ptr(properties.device_name.as_ptr());

      println!("Physical device {}: {}", i, device_name.to_string_lossy());

      // Pretty print the API version
      let api_version = properties.api_version;
      let major = ash::vk::api_version_major(api_version);
      let minor = ash::vk::api_version_minor(api_version);
      let patch = ash::vk::api_version_patch(api_version);
      println!("  • Vulkan API version: {}.{}.{}", major, minor, patch);

      // Print the queue family properties
      let queue_families = instance.get_physical_device_queue_family_properties(*device);
      for (j, q) in queue_families.iter().enumerate() {
        println!("  • Queue family {}: flags = {:?}", j, q.queue_flags);
      }

      // We could also enumerate the device extensions here if we wanted to see what each device supports
      // let extensions = instance.enumerate_device_extension_properties(*device).unwrap();
    }

    // Despite all of that we just pick the first physical device
    physical_devices[index]
  }
}

/// Creates a logical device from the given physical device and instance.
pub fn get_device(instance: &ash::Instance, physical_device: ash::vk::PhysicalDevice, queue_flags: Vec<ash::vk::QueueFlags>) -> ash::Device {
  unsafe {
    println!("Creating logical device...");

    // Step 1: Find a queue family that supports graphics
    let mut family_index: Option<u32> = None;

    let queue_family_properties = instance.get_physical_device_queue_family_properties(physical_device);
    for flag in &queue_flags {
      let index = queue_family_properties
        .iter()
        .enumerate()
        .find(|(_, q)| q.queue_flags.contains(*flag))
        .map(|(index, _)| index)
        .expect(&format!("Failed to find a queue family with flag {:?}", flag)) as u32;
      println!("Found queue family index {} for flag {:?}", index, flag);
      family_index = Some(index);
    }

    let family_index = family_index.expect("Failed to find a suitable queue family") as u32;

    // Step 2: DeviceQueueCreateInfo needed to pick queue family and priority for the logical device
    let queue_priority_list = [1.0f32];
    let queue_create_info = ash::vk::DeviceQueueCreateInfo::default()
      .queue_family_index(family_index)
      .queue_priorities(&queue_priority_list);
    let queue_create_infos = [queue_create_info];

    // Step 3: Create the logical device with the queue create info
    let device_create_info = ash::vk::DeviceCreateInfo::default().queue_create_infos(&queue_create_infos);
    let device = instance.create_device(physical_device, &device_create_info, None).expect("Failed to create logical device");

    println!("Logical device created successfully.");
    device
  }
}
