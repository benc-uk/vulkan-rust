# Vulkan Tutorial but in Rust

Things I want to do:

- Follow the tutorial at https://docs.vulkan.org/tutorial/latest/00_Introduction.html
- Use Rust instead of C/C++
- Use the `ash` crate for Vulkan bindings in Rust
- Use `winit` for window creation and event handling
- Get a basic Vulkan application running in Rust, following the tutorial steps

## 1. Triangle App

This is the minimal Vulkan application that draws a triangle on the screen. It is based on the Vulkan tutorial, but implemented in Rust using `ash` for Vulkan bindings and `winit` for window creation and event handling.

See https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/00_Base_code.html

In this repository the code for this app is in the [triangle/ folder](./triangle).

Links to the tutorial steps as incremental branches:

- [00-setup](../tree/00-setup) - Initial setup of the project, including dependencies and basic window creation.
- [01-instance](../tree/01-instance) - Creating a Vulkan instance and setting up the application info.
