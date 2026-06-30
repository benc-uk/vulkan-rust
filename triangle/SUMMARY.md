# Drawing a Triangle: Step by Step

A complete, step-by-step account of everything this project does to get a triangle on screen, broken into **setup** and **per-frame rendering**, listing every struct created and every Vulkan/ash API call.

Notes:

- Uses `KHR_dynamic_rendering` and `synchronization2`, so there is **no** `VkRenderPass` or `VkFramebuffer` anywhere.
- The triangle's vertex positions and colours come from the Slang vertex shader, so there is no vertex buffer.

## Inventory of Vulkan objects

### Persistent objects (handles we own)

These are the live Vulkan handles created during setup. They each need explicit destruction in reverse creation order (not yet implemented).

| Object                     | Type                             | Count             | Created by                            | Destroyed by                           |
| -------------------------- | -------------------------------- | ----------------- | ------------------------------------- | -------------------------------------- |
| Entry (loader)             | `ash::Entry`                     | 1                 | `ash::Entry::load()`                  | dropped automatically                  |
| Instance                   | `ash::Instance` / `vk::Instance` | 1                 | `entry.create_instance()`             | `instance.destroy_instance()`          |
| Surface                    | `vk::SurfaceKHR`                 | 1                 | `ash_window::create_surface()`        | `surface_loader.destroy_surface()`     |
| Physical device            | `vk::PhysicalDevice`             | 1                 | picked, not created                   | n/a (not owned)                        |
| Logical device             | `ash::Device` / `vk::Device`     | 1                 | `instance.create_device()`            | `device.destroy_device()`              |
| Queue                      | `vk::Queue`                      | 1                 | `device.get_device_queue()`           | n/a (owned by device)                  |
| Swapchain                  | `vk::SwapchainKHR`               | 1                 | `swapchain_loader.create_swapchain()` | `swapchain_loader.destroy_swapchain()` |
| Swapchain images           | `vk::Image`                      | N (per swapchain) | `get_swapchain_images()`              | n/a (owned by swapchain)               |
| Image views                | `vk::ImageView`                  | N (one per image) | `device.create_image_view()`          | `device.destroy_image_view()`          |
| Shader module              | `vk::ShaderModule`               | 1 (both stages)   | `device.create_shader_module()`       | `device.destroy_shader_module()`       |
| Pipeline layout            | `vk::PipelineLayout`             | 1                 | `device.create_pipeline_layout()`     | `device.destroy_pipeline_layout()`     |
| Graphics pipeline          | `vk::Pipeline`                   | 1                 | `device.create_graphics_pipelines()`  | `device.destroy_pipeline()`            |
| Command pool               | `vk::CommandPool`                | 1                 | `device.create_command_pool()`        | `device.destroy_command_pool()`        |
| Command buffers            | `vk::CommandBuffer`              | N (one per image) | `device.allocate_command_buffers()`   | freed with pool                        |
| Present-complete semaphore | `vk::Semaphore`                  | 1                 | `device.create_semaphore()`           | `device.destroy_semaphore()`           |
| Render-complete semaphore  | `vk::Semaphore`                  | 1                 | `device.create_semaphore()`           | `device.destroy_semaphore()`           |
| Draw fence                 | `vk::Fence`                      | 1                 | `device.create_fence()`               | `device.destroy_fence()`               |

### Loader objects (ash extension dispatchers)

Thin ash structs that load extension function pointers. Not Vulkan handles, no destruction needed.

| Object           | Type                          | Loads              |
| ---------------- | ----------------------------- | ------------------ |
| Surface loader   | `ash::khr::surface::Instance` | `VK_KHR_surface`   |
| Swapchain loader | `ash::khr::swapchain::Device` | `VK_KHR_swapchain` |

### Transient `*CreateInfo` / state structs

Stack-built descriptor structs passed into `create_*` / `cmd_*` calls. They do not persist after the call returns. Counts are how many instances are built: `1` once at setup, `N` once per swapchain image, `per frame` rebuilt every rendered frame.

| Struct                                     | Phase       | Count                                      | Built in                           |
| ------------------------------------------ | ----------- | ------------------------------------------ | ---------------------------------- |
| `vk::ApplicationInfo`                      | Instance    | 1                                          | `vulkan::init`                     |
| `vk::InstanceCreateInfo`                   | Instance    | 1                                          | `vulkan::init`                     |
| `vk::DeviceQueueCreateInfo`                | Device      | 1                                          | `vulkan::get_device`               |
| `vk::PhysicalDeviceVulkan11Features`       | Device      | 1                                          | `vulkan::get_device`               |
| `vk::PhysicalDeviceVulkan13Features`       | Device      | 1                                          | `vulkan::get_device`               |
| `vk::DeviceCreateInfo`                     | Device      | 1                                          | `vulkan::get_device`               |
| `vk::SwapchainCreateInfoKHR`               | Swapchain   | 1                                          | `vulkan::create_swapchain`         |
| `vk::Extent2D`                             | Swapchain   | 1 (stored, reused per frame)               | `vulkan::create_swapchain`         |
| `vk::ImageViewCreateInfo`                  | Image views | N (one per image)                          | `vulkan::create_image_view`        |
| `vk::ComponentMapping`                     | Image views | N (one per image)                          | `vulkan::create_image_view`        |
| `vk::ImageSubresourceRange`                | Image views | N (one per image)                          | `vulkan::create_image_view`        |
| `vk::ShaderModuleCreateInfo`               | Shaders     | 1                                          | `vulkan::create_shader_module`     |
| `vk::PipelineShaderStageCreateInfo`        | Shaders     | 2 (vertex, fragment)                       | `vulkan::create_shader_stage_info` |
| `vk::PipelineDynamicStateCreateInfo`       | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineVertexInputStateCreateInfo`   | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineInputAssemblyStateCreateInfo` | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineViewportStateCreateInfo`      | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineRasterizationStateCreateInfo` | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineMultisampleStateCreateInfo`   | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineColorBlendAttachmentState`    | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineColorBlendStateCreateInfo`    | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineLayoutCreateInfo`             | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::PipelineRenderingCreateInfo`          | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::GraphicsPipelineCreateInfo`           | Pipeline    | 1                                          | `vulkan::create_pipeline`          |
| `vk::CommandPoolCreateInfo`                | Commands    | 1                                          | `vulkan::allocate_command_buffers` |
| `vk::CommandBufferAllocateInfo`            | Commands    | 1                                          | `vulkan::allocate_command_buffers` |
| `vk::SemaphoreCreateInfo`                  | Sync        | 2 (present, render complete)               | `app::resumed`                     |
| `vk::FenceCreateInfo`                      | Sync        | 1                                          | `app::resumed`                     |
| `vk::CommandBufferBeginInfo`               | Per-frame   | 1 per frame                                | `record_command_buffer`            |
| `vk::ImageMemoryBarrier2`                  | Per-frame   | 2 per frame (acquire + present transition) | `transition_image_layout`          |
| `vk::ImageSubresourceRange`                | Per-frame   | 2 per frame (one per barrier)              | `transition_image_layout`          |
| `vk::DependencyInfo`                       | Per-frame   | 2 per frame (one per barrier)              | `transition_image_layout`          |
| `vk::RenderingAttachmentInfo`              | Per-frame   | 1 per frame                                | `record_command_buffer`            |
| `vk::ClearValue` / `vk::ClearColorValue`   | Per-frame   | 1 per frame                                | `record_command_buffer`            |
| `vk::RenderingInfo`                        | Per-frame   | 1 per frame                                | `record_command_buffer`            |
| `vk::Rect2D` / `vk::Offset2D`              | Per-frame   | 2 per frame (render area + scissor)        | `record_command_buffer`            |
| `vk::Viewport`                             | Per-frame   | 1 per frame                                | `record_command_buffer`            |
| `vk::SubmitInfo`                           | Per-frame   | 1 per frame                                | `window_event` (RedrawRequested)   |
| `vk::PresentInfoKHR`                       | Per-frame   | 1 per frame                                | `window_event` (RedrawRequested)   |

### Totals

`N` is the number of swapchain images (3 in the worked example below, assuming triple buffering).

| Category                                                      | Distinct kinds | Instances (formula) | Example, N = 3 |
| ------------------------------------------------------------- | -------------- | ------------------- | -------------- |
| Persistent objects (all rows)                                 | 17             | 11 + 3N             | 20             |
| - owned, need explicit destruction                            | 12             | 11 + N              | 14             |
| - not owned (entry, physical device, queue, swapchain images) | 4              | 3 + N               | 6              |
| Loader objects                                                | 2              | 2                   | 2              |
| Transient structs, one-time setup                             | 25             | 27                  | 27             |
| Transient structs, per swapchain image                        | 3              | 3N                  | 9              |
| Transient structs, per frame                                  | 13             | 18 per frame        | 18 / frame     |

Notes:

- Owned objects needing a `destroy_*` call: instance, surface, logical device, swapchain, image views (N), shader module, pipeline layout, pipeline, command pool, 2 semaphores, fence. Command buffers are freed with the pool; swapchain images with the swapchain.
- The 18 per-frame struct instances break down as: 1 begin info, 2 barriers + 2 subresource ranges + 2 dependency infos, 1 rendering attachment, 1 clear value + 1 clear colour value, 1 rendering info, 2 rect2d + 2 offset2d, 1 viewport, 1 submit, 1 present.

## Part A: Application bootstrap (non-Vulkan)

1. `main.rs` creates a winit `EventLoop` via `EventLoop::new()`.
2. Sets control flow with `set_control_flow(ControlFlow::Poll)` then overrides with `ControlFlow::Wait`.
3. Constructs `VulkanApp::new("Vulkan App: Triangle")` and calls `event_loop.run_app(&mut app)`.
4. winit fires `resumed()` (in `app.rs`), which creates the OS window via `event_loop.create_window()` using `Window::default_attributes()` with `LogicalSize`, title, decorations.
5. Extracts the platform handles: `display_handle().as_raw()` (`RawDisplayHandle`) and `window_handle().as_raw()` (`RawWindowHandle`).

## Part B: Vulkan setup (one-time)

### Step 1 — Instance creation (`vulkan::init`)

6. Load the Vulkan loader: `ash::Entry::load()`.
7. Build `vk::ApplicationInfo` (`default()` + `.application_name`, `.application_version`, `.engine_name`, `.engine_version`, `.api_version` via `vk::make_api_version`).
8. Query required instance extensions from the display handle: `ash_window::enumerate_required_extensions(display_handle)` (surface + platform-specific WSI extensions).
9. In debug builds, define validation layers array `[c"VK_LAYER_KHRONOS_validation"]` and collect raw `*const i8` pointers.
10. Build `vk::InstanceCreateInfo` (`.application_info`, `.enabled_extension_names`, `.enabled_layer_names`).
11. Create the instance: `entry.create_instance()` -> `ash::Instance`.

### Step 2 — Surface creation (`vulkan::get_surface`)

12. Create the window surface: `ash_window::create_surface(entry, instance, display, window, None)` -> `vk::SurfaceKHR`.
13. Create the surface extension loader: `surface::Instance::new(entry, instance)` (`ash::khr::surface::Instance`).

### Step 3 — Physical + logical device (`vulkan::get_device`)

14. Enumerate GPUs: `instance.enumerate_physical_devices()`.
15. For each device query queue families: `instance.get_physical_device_queue_family_properties()`.
16. For each family check graphics support (`queue_flags.contains(vk::QueueFlags::GRAPHICS)`) and present support (`surface_loader.get_physical_device_surface_support()`), pick the first matching `(physical_device, queue_family_index)`.
17. Read device properties for logging: `instance.get_physical_device_properties()` (device name, API version).
18. Build `vk::DeviceQueueCreateInfo` (`.queue_family_index`, `.queue_priorities = [1.0]`).
19. List required device extensions: `[swapchain::NAME]` (`VK_KHR_swapchain`).
20. Enable features: `vk::PhysicalDeviceVulkan13Features` (`.dynamic_rendering(true)`, `.synchronization2(true)`) and `vk::PhysicalDeviceVulkan11Features` (`.shader_draw_parameters(true)`).
21. Build `vk::DeviceCreateInfo` (`.queue_create_infos`, `.enabled_extension_names`, `.push_next(vk13)`, `.push_next(vk11)`).
22. Create logical device: `instance.create_device()` -> `ash::Device`.

### Step 4 — Swapchain (`vulkan::create_swapchain`)

23. Query supported surface formats: `surface_loader.get_physical_device_surface_formats()`; pick `B8G8R8A8_SRGB` or fallback to first.
24. Query present modes: `surface_loader.get_physical_device_surface_present_modes()`; pick `MAILBOX` or fallback to `FIFO`.
25. Query surface capabilities: `surface_loader.get_physical_device_surface_capabilities()`; resolve `vk::Extent2D` (use current extent or requested size when `u32::MAX`).
26. Compute desired image count (`min_image_count + 1`, clamped to `max_image_count`).
27. Build `vk::SwapchainCreateInfoKHR` (`.surface`, `.min_image_count`, `.image_format`, `.image_color_space`, `.image_extent`, `.image_array_layers(1)`, `.image_usage(COLOR_ATTACHMENT)`, `.image_sharing_mode(EXCLUSIVE)`, `.pre_transform`, `.composite_alpha(OPAQUE)`, `.present_mode`, `.clipped(true)`).
28. Create the swapchain loader: `swapchain::Device::new(instance, device)` (`ash::khr::swapchain::Device`).
29. Create the swapchain: `swapchain_loader.create_swapchain()` -> `vk::SwapchainKHR`.
30. Retrieve swapchain images: `swapchain_loader.get_swapchain_images()` -> `Vec<vk::Image>`.

### Step 5 — Image views (`vulkan::create_image_view`, one per image)

31. Build `vk::ImageViewCreateInfo` (`.image`, `.view_type(TYPE_2D)`, `.format`, `.components` via `vk::ComponentMapping` all `IDENTITY`, `.subresource_range` via `vk::ImageSubresourceRange` with `COLOR` aspect, 1 mip, 1 layer).
32. Create the view: `device.create_image_view()` -> `vk::ImageView`.

### Step 6 — Shader modules and stages (`vulkan::create_shader_module` / `create_shader_stage_info`)

33. SPIR-V is compiled at build time from `shaders/shader.slang` and embedded via `include_bytes!(OUT_DIR/shader.spv)`.
34. Read SPIR-V: `ash::util::read_spv()` over a `Cursor`.
35. Build `vk::ShaderModuleCreateInfo` (`.code`) and create: `device.create_shader_module()` -> `vk::ShaderModule` (single module holds both entry points).
36. Build two `vk::PipelineShaderStageCreateInfo`: vertex stage (`.module`, `.stage(VERTEX)`, `.name(c"vertMain")`) and fragment stage (`.stage(FRAGMENT)`, `.name(c"fragMain")`).

### Step 7 — Graphics pipeline (`vulkan::create_pipeline`)

37. `vk::PipelineDynamicStateCreateInfo` with dynamic states `[VIEWPORT, SCISSOR]`.
38. `vk::PipelineVertexInputStateCreateInfo` (empty, no vertex buffers).
39. `vk::PipelineInputAssemblyStateCreateInfo` (`.topology(TRIANGLE_LIST)`).
40. `vk::PipelineViewportStateCreateInfo` (`.viewport_count(1)`, `.scissor_count(1)`).
41. `vk::PipelineRasterizationStateCreateInfo` (`.polygon_mode(FILL)`, `.cull_mode(BACK)`, `.front_face(CLOCKWISE)`, `.line_width(1.0)`).
42. `vk::PipelineMultisampleStateCreateInfo` (`.rasterization_samples(TYPE_1)`, `.sample_shading_enable(false)`).
43. `vk::PipelineColorBlendAttachmentState` (`.color_write_mask(RGBA)`) and `vk::PipelineColorBlendStateCreateInfo` (`.attachments`).
44. `vk::PipelineLayoutCreateInfo` (empty) -> `device.create_pipeline_layout()` -> `vk::PipelineLayout`.
45. `vk::PipelineRenderingCreateInfo` (`.color_attachment_formats = [format]`) for dynamic rendering (no render pass).
46. Assemble `vk::GraphicsPipelineCreateInfo` (`.stages`, all the fixed-function state structs above, `.layout`, `.dynamic_state`, `.push_next(rendering_info)`).
47. Create the pipeline: `device.create_graphics_pipelines(PipelineCache::null(), ...)` -> `vk::Pipeline`. (No render pass, no framebuffers, because dynamic rendering is used.)

### Step 8 — Command pool and buffers (`vulkan::allocate_command_buffers`)

48. `vk::CommandPoolCreateInfo` (`.queue_family_index`, `.flags(RESET_COMMAND_BUFFER)`) -> `device.create_command_pool()` -> `vk::CommandPool`.
49. `vk::CommandBufferAllocateInfo` (`.command_pool`, `.command_buffer_count` = image count) -> `device.allocate_command_buffers()` -> `Vec<vk::CommandBuffer>` (one per swapchain image).

### Step 9 — Queue and synchronization objects (back in `resumed`)

50. Retrieve the graphics queue: `device.get_device_queue(qf_index, 0)` -> `vk::Queue`.
51. Create `present_complete_semaphore`: `device.create_semaphore(vk::SemaphoreCreateInfo::default())` -> `vk::Semaphore`.
52. Create `render_complete_semaphore`: second `device.create_semaphore()` -> `vk::Semaphore`.
53. Create the draw fence: `device.create_fence(vk::FenceCreateInfo with flags(SIGNALED))` -> `vk::Fence` (starts signalled so the first frame doesn't block).
54. Store every handle in the `VulkanApp` struct for later use.

## Part C: Rendering a single frame (`window_event` -> `RedrawRequested`)

55. Wait for the previous frame: `device.wait_for_fences(&[draw_fence], true, u64::MAX)`.
56. Reset the fence: `device.reset_fences(&[draw_fence])`.
57. Acquire the next swapchain image: `swapchain_device.acquire_next_image(swapchain, u64::MAX, present_complete_semaphore, Fence::null())` -> `image_index` (signals `present_complete_semaphore` when ready).

### Record the command buffer (`record_command_buffer`)

58. Begin recording: `vk::CommandBufferBeginInfo::default()` -> `device.begin_command_buffer()`.
59. Transition image `UNDEFINED -> COLOR_ATTACHMENT_OPTIMAL` via `transition_image_layout`, which builds `vk::ImageSubresourceRange`, `vk::ImageMemoryBarrier2` (src/dst stage+access masks, queue family `IGNORED`, image, subresource), and `vk::DependencyInfo`, then calls `device.cmd_pipeline_barrier2()` (synchronization2).
60. Build `vk::RenderingAttachmentInfo` (`.image_view`, `.image_layout(COLOR_ATTACHMENT_OPTIMAL)`, `.load_op(CLEAR)`, `.store_op(STORE)`, `.clear_value` = black via `vk::ClearValue`/`vk::ClearColorValue`).
61. Build `vk::RenderingInfo` (`.render_area` via `vk::Rect2D`/`vk::Offset2D`, `.layer_count(1)`, `.color_attachments`).
62. Begin dynamic rendering: `device.cmd_begin_rendering()`.
63. Bind the pipeline: `device.cmd_bind_pipeline(GRAPHICS, pipeline)`.
64. Set the dynamic viewport: build `vk::Viewport` -> `device.cmd_set_viewport()`.
65. Set the dynamic scissor: build `vk::Rect2D` -> `device.cmd_set_scissor()`.
66. Draw: `device.cmd_draw(command_buffer, 3, 1, 0, 0)` (3 vertices, 1 instance; vertex positions/colours come from the Slang vertex shader, no vertex buffer).
67. End rendering: `device.cmd_end_rendering()`.
68. Transition image `COLOR_ATTACHMENT_OPTIMAL -> PRESENT_SRC_KHR` via a second `transition_image_layout`/`cmd_pipeline_barrier2`.
69. Finish recording: `device.end_command_buffer()`.

### Submit and present

70. Build `vk::SubmitInfo` (`.wait_semaphores = [present_complete_semaphore]`, `.wait_dst_stage_mask = [COLOR_ATTACHMENT_OUTPUT]`, `.command_buffers`, `.signal_semaphores = [render_complete_semaphore]`).
71. Submit to the queue: `device.queue_submit(queue, &[submit_info], draw_fence)` (signals the fence when done).
72. Build `vk::PresentInfoKHR` (`.wait_semaphores = [render_complete_semaphore]`, `.swapchains`, `.image_indices`).
73. Present: `swapchain_device.queue_present(queue, &present_info)`.
74. Request another frame: `window.request_redraw()`, which loops back to step 55.

### Window lifecycle

75. `WindowEvent::CloseRequested` calls `event_loop.exit()` to end the loop.
