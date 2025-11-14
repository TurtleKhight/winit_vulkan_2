mod render_context;
mod vulkan;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

use crate::{render_context::RenderContext, vulkan::VulkanContext};

#[macro_export]
macro_rules! msg {
    ($val:expr) => {{
        let val = &$val;
        print!("[{}:{}:{}] {:?}", file!(), line!(), column!(), val);
    }};
}

#[macro_export]
macro_rules! msgln {
    ($val:expr) => {{
        let val = &$val;
        println!("[{}:{}:{}] {:?}", file!(), line!(), column!(), val);
    }};
}

struct App {
    vk_ctx: VulkanContext,
    r_ctx: Option<RenderContext>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let r_ctx = RenderContext::new(window, &self.vk_ctx);
        self.r_ctx = Some(r_ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                msgln!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let r_ctx = self.r_ctx.as_mut().unwrap();
                r_ctx.recreate_swapchain = true;
            }
            WindowEvent::RedrawRequested => {
                let r_ctx = self.r_ctx.as_mut().unwrap();
                r_ctx.render(&self.vk_ctx);
                r_ctx.window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let vk_ctx = VulkanContext::new(&event_loop);
    let mut app = App {
        vk_ctx,
        r_ctx: None,
    };
    let _ = event_loop.run_app(&mut app);
}
