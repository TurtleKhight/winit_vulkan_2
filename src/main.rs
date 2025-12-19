mod game;
mod gui;
mod input;
mod render_context;
mod sysinfo;
mod vulkan;

use std::sync::Arc;

use nalgebra::Vector2;
use winit::{
    application::ApplicationHandler,
    event::{MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, WindowAttributes, WindowId},
};

use crate::{
    game::Game,
    input::{KeyboardBinding, MouseBinding},
    render_context::{RenderConfig, RenderContext},
    sysinfo::SysInfo,
    vulkan::{VulkanConfig, VulkanContext},
};

#[macro_export]
macro_rules! msg {
    ($fmt:literal $(, $args:tt)*) => {{
        print!("[{}:{}:{}] ", file!(), line!(), column!());
        print!($fmt $(, $args)*);
    }};
}

#[macro_export]
macro_rules! msgln {
    ($fmt:literal $(, $args:tt)*) => {{
        print!("[{}:{}:{}] ", file!(), line!(), column!());
        println!($fmt $(, $args)*);
    }};
}

struct App {
    vk_ctx: VulkanContext,
    ren_ctx: Option<RenderContext>,

    game: Game,
    last_frame: std::time::Instant,
    last_render: std::time::Instant,
    dt: f32,

    sysinfo: SysInfo,

    keyboard_input: KeyboardBinding,
    mouse_input: MouseBinding,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let ren_ctx = RenderContext::new(window, &self.vk_ctx, RenderConfig::default(), None);
        self.ren_ctx = Some(ren_ctx);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if self.mouse_input.down(0) {
                let delta = Vector2::new(delta.0 as f32, delta.1 as f32);
                self.game
                    .camera_controller
                    .drag_camera(&mut self.game.camera, delta);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let ren_ctx = self.ren_ctx.as_mut().unwrap();
        ren_ctx.gui.input(&event);

        match event {
            WindowEvent::CloseRequested => {
                msgln!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let size = Vector2::new(size.width, size.height);
                ren_ctx.recreate_swapchain = true;
                self.game.camera.resize(size);
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.keyboard_input.reset();
                    self.mouse_input.reset();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !ren_ctx.gui.ctx.io().want_capture_keyboard {
                    if event.state.is_pressed() {
                        self.keyboard_pressed(&event_loop, &event.physical_key);
                    } else {
                        self.keyboard_released(&event_loop, &event.physical_key);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if !ren_ctx.gui.ctx.io().want_capture_mouse {
                    if state.is_pressed() {
                        self.mouse_pressed(button);
                    } else {
                        self.mouse_released(button);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.dt = self.last_frame.elapsed().as_secs_f32();
                self.last_frame = std::time::Instant::now();
                self.keyboard_down();
                self.do_ui();
                self.sysinfo.update(self.dt);
                let ren_ctx: &mut RenderContext = self.ren_ctx.as_mut().unwrap();
                while !ren_ctx.config.should_render(self.last_render) {}
                self.last_render = std::time::Instant::now();
                ren_ctx.renderer.update(self.dt, &self.game, &self.vk_ctx);
                ren_ctx.render(&self.vk_ctx);
                ren_ctx.window.request_redraw();
            }
            _ => (),
        }
    }
}
impl App {
    fn keyboard_pressed(&mut self, event_loop: &ActiveEventLoop, physical_key: &PhysicalKey) {
        let ren_ctx = self.ren_ctx.as_mut().unwrap();

        match physical_key {
            PhysicalKey::Code(KeyCode::Escape) => {
                event_loop.exit();
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::F11) => {
                let window = ren_ctx.window.clone();
                match window.fullscreen() {
                    Some(_) => {
                        window.set_fullscreen(None);
                    }
                    None => {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                }
            }
            _ => {}
        }
        match physical_key {
            PhysicalKey::Code(key) => {
                self.keyboard_input.set(*key as usize);
            }
            _ => {}
        }
    }

    fn keyboard_released(&mut self, event_loop: &ActiveEventLoop, physical_key: &PhysicalKey) {
        match physical_key {
            PhysicalKey::Code(key) => {
                self.keyboard_input.unset(*key as usize);
            }
            _ => {}
        }
    }

    fn keyboard_down(&mut self) {
        self.game.camera_controller.move_camera(
            &mut self.game.camera,
            &self.keyboard_input,
            self.dt as f32,
        );
    }

    fn mouse_pressed(&mut self, button: MouseButton) {
        let ren_ctx = self.ren_ctx.as_mut().unwrap();
        match button {
            MouseButton::Left => {
                let window = ren_ctx.window.clone();
                window.set_cursor_visible(false);
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
            }
            _ => {}
        }

        match button {
            MouseButton::Left => self.mouse_input.set(0),
            MouseButton::Right => self.mouse_input.set(1),
            MouseButton::Middle => self.mouse_input.set(2),
            MouseButton::Back => self.mouse_input.set(3),
            MouseButton::Forward => self.mouse_input.set(4),
            MouseButton::Other(i) => self.mouse_input.set(5 + i as usize),
        }
    }

    fn mouse_released(&mut self, button: MouseButton) {
        let ren_ctx = self.ren_ctx.as_mut().unwrap();
        match button {
            MouseButton::Left => {
                let window = ren_ctx.window.clone();
                window.set_cursor_visible(true);
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            }
            _ => {}
        }
        match button {
            MouseButton::Left => self.mouse_input.unset(0),
            MouseButton::Right => self.mouse_input.unset(1),
            MouseButton::Middle => self.mouse_input.unset(2),
            MouseButton::Back => self.mouse_input.unset(3),
            MouseButton::Forward => self.mouse_input.unset(4),
            MouseButton::Other(i) => self.mouse_input.unset(5 + i as usize),
        }
    }

    fn do_ui(&mut self) {
        let ren_ctx = self.ren_ctx.as_mut().unwrap();
        ren_ctx
            .gui
            .ui(|ui| {
                ui.window("ImGui Window ").build(|| {
                    self.sysinfo.ui(ui);
                    self.vk_ctx.config.ui(ui);
                    ren_ctx
                        .config
                        .ui(ui, &ren_ctx.swapchain, &mut ren_ctx.recreate_swapchain);
                    self.game.camera.ui(&ui);
                });
            })
            .unwrap();
        ren_ctx.gui.ctx.io_mut().font_global_scale = ren_ctx.config.gui_scale();
        if self.vk_ctx.config.needs_reset() {
            self.reset_renderer();
        }
    }

    fn reset_renderer(&mut self) {
        if let Some(ren) = self.ren_ctx.take() {
            let window = ren.window.clone();
            // TODO: configs dont need to be clone
            self.vk_ctx = VulkanContext::from_instance(
                &window,
                self.vk_ctx.instance.clone(),
                self.vk_ctx.config.clone(),
            );
            let (config, gui) = {
                let ren = ren;

                let config = ren.config;
                let mut gui = ren.gui;
                let _ = gui.ctx.render();
                gui.renderer = None;
                (config, gui)
            };
            let render_context = RenderContext::new(window, &self.vk_ctx, config, Some(gui));

            self.ren_ctx = Some(render_context);
            self.vk_ctx.config.clear_needs_reset();
        }
    }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        vk_ctx: VulkanContext::new(&event_loop, VulkanConfig::default()),
        ren_ctx: None,

        game: Game::default(),
        dt: 1.0,
        last_frame: std::time::Instant::now(),
        last_render: std::time::Instant::now(),

        sysinfo: SysInfo::new(),

        keyboard_input: KeyboardBinding::new(),
        mouse_input: MouseBinding::new(),
    };
    let _ = event_loop.run_app(&mut app);
}
