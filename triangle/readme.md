# Ash, Winit, Vulkan - Triangle App

## Step: 00 - Setup

This is little more than a basic Rust project with the ash and winit dependencies added. There's no Vulkan code yet, but the project is set up and ready to go.

This branch equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/00_Base_code.html

## Step: 01 - Creating an instance

Add the barebone Vulkan code to create a Vulkan instance. This is the first step in getting a Vulkan application running.

This branch equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/01_Instance.html

## Step: 02 - Validation Layers

Gotcha here, first install the `vulkan-validationlayers` package:

```sh
sudo apt install vulkan-validationlayers
```

This branch equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/02_Validation_layers.html

## Step: 03 - Physical Devices

Refactored the boilerplate Vulkan code into a `vulkan` module, and added code to enumerate physical devices and queue families.
This is the first step in selecting a GPU to use for rendering.

This branch equates to the tutorial step at

https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/00_Setup/03_Physical_devices_and_queue_families.html
