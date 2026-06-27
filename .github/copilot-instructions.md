# Project Guidelines

Learning project: working through the [Vulkan Tutorial](https://docs.vulkan.org/tutorial/latest/00_Introduction.html) in Rust instead of C/C++. The point is to learn, so explain non-obvious Vulkan steps briefly rather than just emitting code.

## Guidelines

- NEVER update code unless the user has explicitly requested it. If the user asks for a change, explain what you are doing and why.
- Rarely provide code snippets unless the user has explicitly requested it. If the user asks for a code snippet, explain what you are doing and why.
- Guidance is the key goal, not jumping straight to code. If the user asks for a code snippet, explain what you are doing and why.

## Stack

- `ash` for Vulkan bindings (thin, unsafe FFI, mirrors the C API)
- `winit` for windowing and the event loop
- Rust edition 2024
- This is not a mobile application so iOS/Android is never a concern (even though `winit` supports them). Focus on desktop platforms (Windows, Linux, macOS).

## Code Style

- Idiomatic Rust first. Prefer ownership, borrowing and the type system over C++ habits.
- Clean up Vulkan handles in `Drop` impls, in reverse creation order. Do not call it RAII, it is just `Drop`.
- Keep `unsafe` blocks as small as possible and add a short `// SAFETY:` note explaining the invariant being upheld.
- Return `Result` and use `?` for fallible setup. Avoid `unwrap()`/`expect()` outside early prototypes and tests.
- No needless abstractions or wrappers until the tutorial actually needs them.

## Vulkan / ash Patterns

- Enable validation layers in debug builds and wire up a debug messenger early. Treat validation errors as bugs to fix, not noise.
- Use `ash::vk` builder structs (`vk::...::default()` with setters); make sure any referenced slices/strings outlive the `create_*` call.
- Follow the tutorial's ordering: instance, surface, physical device, logical device + queues, swapchain, image views, render pass, pipeline, framebuffers, command pool/buffers, sync objects.
- Pick queue families and swapchain settings explicitly; do not hardcode indices.

## Build and Run

- `cargo build` / `cargo run`
- `cargo clippy` should stay clean; address warnings rather than silencing them.
- `cargo fmt` before considering a change done.

## Conventions

- Track tutorial progress in [readme.md](../readme.md).
- One tutorial chapter per logical change; keep `main.rs` readable as it grows and split into modules when a stage gets large.
