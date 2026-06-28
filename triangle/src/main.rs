use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod vulkan;

fn main() {
  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(ControlFlow::Poll);
  event_loop.set_control_flow(ControlFlow::Wait);

  let mut app = app::VulkanApp::new("Vulkan App: Triangle").unwrap();

  event_loop.run_app(&mut app).unwrap();
}
