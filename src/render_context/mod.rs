use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        PrimaryCommandBufferAbstract,
    },
    image::{Image, ImageUsage},
    pipeline::graphics::viewport::Viewport,
    swapchain::{
        Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
        acquire_next_image,
    },
    sync::GpuFuture,
};
use winit::window::Window;

use crate::{gui::Gui, msgln, vulkan::VulkanContext};

mod config;
mod renderer;

pub use config::RenderConfig;
use renderer::Renderer;

pub struct RenderContextInvariant {
    pub gui: Gui,
    pub config: RenderConfig,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    pub swapchain_viewport: Viewport,

    pub renderer: Renderer,
    pub gui: Gui,
    pub config: RenderConfig,
}
impl RenderContext {
    pub fn new(
        window: Arc<Window>,
        vk_ctx: &VulkanContext,
        mut config: RenderConfig,
        gui: Option<Gui>,
    ) -> Self {
        let surface = Surface::from_window(vk_ctx.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();
        let (swapchain, images) = {
            let surface_capabilities = vk_ctx
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let surface_formats = vk_ctx
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap();

            let (image_format, _) = surface_formats[0];

            let modes = vk_ctx
                .device
                .physical_device()
                .surface_present_modes(&surface, SurfaceInfo::default())
                .unwrap();
            config.fallback_present_mode(&modes);

            Swapchain::new(
                vk_ctx.device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    // image_format: vulkano::format::Format::B8G8R8A8_SRGB,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    present_mode: config.present_mode(),
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),

                    ..Default::default()
                },
            )
            .unwrap()
        };

        let swapchain_viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [window_size.width as f32, window_size.height as f32],
            depth_range: 0.0..=1.0,
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            vk_ctx.cmd_alloc.clone(),
            vk_ctx.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let renderer = Renderer::new(&vk_ctx, &mut builder, &images, swapchain.image_format());

        let mut gui = if let Some(gui) = gui {
            gui
        } else {
            let gui = Gui::new(window.clone());
            gui
        };
        gui.new_renderer(
            vk_ctx.device.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
            vulkano::render_pass::Subpass::from(renderer.get_final_pass(), 0).unwrap(),
            window.clone(),
        );
        gui.renderer
            .as_mut()
            .unwrap()
            .reload_font_textures(&mut gui.ctx, &mut builder);
        let cb = builder.build().unwrap();
        cb.execute(vk_ctx.queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        Self {
            window,
            swapchain,
            images,
            recreate_swapchain: false,
            previous_frame_end: Some(vulkano::sync::now(vk_ctx.device.clone()).boxed()),

            swapchain_viewport,

            renderer,
            gui,
            config,
        }
    }

    pub fn render(&mut self, vk_ctx: &VulkanContext) {
        let window_size = self.window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.into(),
                    present_mode: self.config.present_mode(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;
            self.images = new_images;

            self.swapchain_viewport.extent = [window_size.width as f32, window_size.height as f32];
            self.renderer.resize_swap(&self.images);

            self.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            AutoCommandBufferBuilder::primary(
                vk_ctx.cmd_alloc.clone(),
                vk_ctx.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

        // ===================================================================== Start of Render

        // ===================================================================== Forward Pass

        self.renderer.render_forward_render_pass(&mut builder);

        // ===================================================================== Final Pass

        self.renderer.final_render_pass(
            &mut self.gui,
            &mut builder,
            image_index as usize,
            self.swapchain_viewport.clone(),
        );

        // ===================================================================== End of render
        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(vk_ctx.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vk_ctx.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(mut future) => {
                future.cleanup_finished();
                // future.wait(None).unwrap();
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(vulkano::sync::now(vk_ctx.device.clone()).boxed());
            }
            Err(e) => {
                panic!("failed to flush future: {e}");
            }
        }
    }
}
