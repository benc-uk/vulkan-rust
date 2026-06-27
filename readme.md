# Vulkan Tutorial but in Rust

Maybe I've not made up my mind yet.

Things I want to do:

- Follow the tutorial at https://docs.vulkan.org/tutorial/latest/00_Introduction.html
- Use Rust instead of C/C++
- Use the `ash` crate for Vulkan bindings in Rust
- Use `winit` for window creation and event handling
- Get a basic Vulkan application running in Rust, following the tutorial steps

## Triangle App

This is the minimal Vulkan application that draws a triangle on the screen. It is based on the Vulkan tutorial, but implemented in Rust using `ash` for Vulkan bindings and `winit` for window creation and event handling.

See https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/00_Base_code.html

In this repository the code for this app is in the [`triangle` folder](./triangle).
