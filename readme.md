# Vulkan Tutorial but in Rust

Things I want to do:

- Follow the tutorial at https://docs.vulkan.org/tutorial/latest/00_Introduction.html
- Use Rust instead of C/C++
- Use the `ash` crate for Vulkan bindings in Rust
- Use `winit` for window creation and event handling
- Get a basic Vulkan application running in Rust, following the tutorial steps

## 1. Triangle App

This is the minimal Vulkan application that draws a single triangle on the screen. It is based on the official Vulkan tutorial.
Drawing a single triangle doesn't seem like much, but there's a lot of setup involved in getting even to that point!

See https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/00_Base_code.html

In this repository the code for this app is in the [triangle/ folder](./triangle).

Links to the tutorial steps as incremental tags:

### Setup

- [1.0.0-setup](https://github.com/benc-uk/vulkan-rust/tree/1.0.0-setup/triangle) - Initial setup of the project, including dependencies and basic window creation.
- [1.0.1-instance](https://github.com/benc-uk/vulkan-rust/tree/1.0.1-instance/triangle) - Creating a Vulkan instance and setting up the application info.
- [1.0.2-validation-layers](https://github.com/benc-uk/vulkan-rust/tree/1.0.2-validation-layers/triangle) - Adding validation layers to help catch mistakes in Vulkan usage.
- [1.0.3-phys-device](https://github.com/benc-uk/vulkan-rust/tree/1.0.3-phys-device/triangle) - Enumerating physical devices and queue families.
- [1.0.4-logical-device](https://github.com/benc-uk/vulkan-rust/tree/1.0.4-logical-device/triangle) - Creating a logical device with a graphics queue.

### Presentation

- [1.1.0-surface](https://github.com/benc-uk/vulkan-rust/tree/1.1.0-surface/triangle) - Creating a Vulkan surface for the window.
- [1.1.1-swapchain](https://github.com/benc-uk/vulkan-rust/tree/1.1.1-swapchain/triangle) - Setting up the swapchain for presenting images to the window.
- [1.1.2-image-views](https://github.com/benc-uk/vulkan-rust/tree/1.1.2-image-views/triangle) - Creating image views for the swapchain images.

### Pipeline

- [1.2.0-shader-modules](https://github.com/benc-uk/vulkan-rust/tree/1.2.0-shader-modules/triangle) - Creating shader modules from SPIR-V, compiling Slang shaders, and beginning to set up the graphics pipeline.
- [1.2.1-fixed-func](https://github.com/benc-uk/vulkan-rust/tree/1.2.1-fixed-func/triangle) - Creating the fixed-function pipeline stages and the pipeline layout.
- [1.2.2-pipeline](https://github.com/benc-uk/vulkan-rust/tree/1.2.2-pipeline/triangle) - Creating the graphics pipeline with dynamic rendering. This combines several of the official tutorial steps into one, e.g. Render Passes/Dynamic Rendering and Conclusion

### Drawing

- [1.3.0-command-buffers](https://github.com/benc-uk/vulkan-rust/tree/1.3.0-command-buffers/triangle) - Recording command buffers.
- [1.3.1-rendering](https://github.com/benc-uk/vulkan-rust/tree/1.3.1-rendering/triangle) - Submitting command buffers and presenting the rendered image. Finally, we have a triangle on the screen!
- [1.3.2-in-flight](https://github.com/benc-uk/vulkan-rust/tree/1.3.2-in-flight/triangle) - Managing multiple frames in flight to improve performance and handle window resizing with a swapchain recreation.
