# Ash, Winit, Vulkan - Triangle App

# Setup

## Step: 00 - Setup

This is little more than a basic Rust project with the ash and winit dependencies added.
There's no Vulkan code yet, but the project is set up and ready to go.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/00_Base_code.html

## Step: 01 - Creating an instance

Add the barebone Vulkan code to create a Vulkan instance. This is the first step in getting a Vulkan application running.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/01_Instance.html

## Step: 02 - Validation Layers

Gotcha here, first install the `vulkan-validationlayers` package:

```sh
sudo apt install vulkan-validationlayers
```

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/02_Validation_layers.html

## Step: 03 - Physical Devices

Refactored the boilerplate Vulkan code into a `vulkan` module, and added code to enumerate physical devices and queue families.
This is the first step in selecting a GPU to use for rendering.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/03_Physical_devices_and_queue_families.html

## Step: 04 - Logical Device

Added code to create a logical device from the selected physical device, and to get a graphics queue from that logical device.
This is the first step in setting up the rendering pipeline.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/04_Logical_device_and_queues.html

# Presentation

## Step: 05 - Surface

Added code to create a Vulkan surface for the window, this also includes a refactor of the `get_device` & `get_physical_device` functions to
a single function that returns the physical device, logical device, and graphics queue family index. This is the first step in setting up the presentation pipeline.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/01_Presentation/00_Window_surface.html

## Step: 06 - Swapchain

Swapchain creation is the next step in setting up the presentation pipeline. This includes creating a swapchain, and getting the swapchain images. Which is a surpringly large amount of code for a small step, but it's another step in the Vulkan tutorial.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/01_Presentation/01_Swap_chain.html

## Step: 07 - Image Views

Image views are the next step in setting up the presentation pipeline. This includes creating an image view for each swapchain image. Which unlike the swapchain creation is a small amount of code.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/01_Presentation/02_Image_views.html

# Pipeline

## Step: 08 - Shader Modules

Adds code to create shader modules from SPIR-V bytecode, and an introduction to Slang and building/compiling shaders. This is the first step in setting up the graphics pipeline.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/01_Shader_modules.html

## Step: 09 - Fixed Function

Configures the various fixed-function stages of the graphics pipeline, including the input assembly, viewport, rasterizer, and color blending. Also sets up the pipeline layout.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/02_Fixed_functions.html

## Step: 10 - Pipeline

Build the graphics pipeline with dynamic rendering, which combines two of the official tutorial steps into one, e.g. Render Passes/Dynamic Rendering and Conclusion.

This tag equates to the tutorial steps at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/03_Render_passes.html  
https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/04_Conclusion.html

# Drawing

## Step: 11 - Command Buffers

Create command buffers and record the commands to draw a triangle. Also add helper to transition the swapchain image layouts, and to submit the command buffers to the graphics queue.

This tag equates to the tutorial steps at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/01_Command_buffers.html

## Step: 12 - Rendering & Presentation

Finally, implement the rendering loop to submit command buffers and present the rendered image to the window. Finally, we have a triangle on the screen!

This tag equates to the tutorial steps at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/02_Rendering_and_presentation.html

## Step: 13 - In Flight Frames + Swapchain Recreation

We combine the final two steps of the official 'drawing a triangle' tutorial into one, to manage multiple frames in flight to improve performance and handle window resizing with a swapchain recreation.

This tag equates to the tutorial steps at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/03_Frames_in_flight.html  
https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/04_Swap_chain_recreation.html
