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

## Step: 05 - Surface

Added code to create a Vulkan surface for the window, this also includes a refactor of the `get_device` & `get_physical_device` functions to
a single function that returns the physical device, logical device, and graphics queue family index. This is the first step in setting up the presentation pipeline.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/01_Presentation/00_Window_surface.html

## Step: 06 - Swapchain

Swapchain creation is the next step in setting up the presentation pipeline. This includes creating a swapchain, and getting the swapchain images. Which is a surpringly large amount of code for a small step, but it's another step in the Vulkan tutorial.

This tag equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/01_Presentation/01_Swap_chain.html
