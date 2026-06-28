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

- [1.0.0-setup](https://github.com/benc-uk/vulkan-rust/tree/1.0.0-setup/triangle) - Initial setup of the project, including dependencies and basic window creation.
- [1.0.1-instance](https://github.com/benc-uk/vulkan-rust/tree/1.0.1-instance/triangle) - Creating a Vulkan instance and setting up the application info.
- [1.0.2-validation-layers](https://github.com/benc-uk/vulkan-rust/tree/1.0.2-validation-layers/triangle) - Adding validation layers to help catch mistakes in Vulkan usage.
- [1.0.3-phys-device](https://github.com/benc-uk/vulkan-rust/tree/1.0.3-phys-device/triangle) - Enumerating physical devices and queue families.
- [1.0.4-logical-device](https://github.com/benc-uk/vulkan-rust/tree/1.0.4-logical-device/triangle) - Creating a logical device with a graphics queue.
