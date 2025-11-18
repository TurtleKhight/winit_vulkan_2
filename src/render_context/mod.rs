mod draw_texture;
mod render_pass;
mod renderer;

use std::sync::Arc;

use vulkano::{
    Validated, VulkanError,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
    descriptor_set::{DescriptorSet, layout::DescriptorSetLayout},
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageUsage,
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView,
    },
    memory::allocator::AllocationCreateInfo,
    pipeline::{
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, graphics::viewport::Viewport,
    },
    swapchain::{
        PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
        acquire_next_image,
    },
    sync::GpuFuture,
};
use winit::window::Window;

use crate::{
    render_context::{
        render_pass::{final_single_pass::FinalSingleRenderPass, single_pass::SingleRenderPass},
        renderer::Renderer,
    },
    vulkan::VulkanContext,
};

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,

    pub forward_images: Vec<Arc<ImageView>>,
    pub forward_render_pass: SingleRenderPass,
    pub forward_viewport: Viewport,
    pub final_render_pass: FinalSingleRenderPass,
    pub final_viewport: Viewport,

    pub renderer: Renderer,

    blit_texture_layout: Arc<DescriptorSetLayout>,
    blit_texture_pipeline: Arc<GraphicsPipeline>,
    blit_desc: Arc<DescriptorSet>,
}
impl RenderContext {
    pub fn new(window: Arc<Window>, vk_ctx: &VulkanContext) -> Self {
        let surface = Surface::from_window(vk_ctx.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();
        let (swapchain, images) = {
            let surface_capabilities = vk_ctx
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let (image_format, _) = vk_ctx
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                vk_ctx.device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    // image_format: vulkano::format::Format::R8G8B8A8_SRGB,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    present_mode: PresentMode::Fifo,
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

        let final_viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [window_size.width as f32, window_size.height as f32],
            depth_range: 0.0..=1.0,
        };
        let extent = [256, 256, 1];
        let forward_viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };
        let depth = ImageView::new_default(
            Image::new(
                vk_ctx.mem_alloc.clone(),
                ImageCreateInfo {
                    extent,
                    format: Format::D16_UNORM,
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let albedo = ImageView::new_default(
            Image::new(
                vk_ctx.mem_alloc.clone(),
                ImageCreateInfo {
                    extent,
                    format: Format::R8G8B8A8_UNORM,
                    usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let forward_images = vec![albedo, depth];
        let final_render_pass = FinalSingleRenderPass::new(
            &vk_ctx.device,
            &images,
            swapchain.image_format(),
            &[],
            false,
        );
        let forward_render_pass =
            SingleRenderPass::new(&vk_ctx.device, forward_images.clone(), true);

        let renderer = Renderer::new(&vk_ctx, forward_render_pass.render_pass.clone());

        let blit_texture_layout = draw_texture::set_layout(vk_ctx.device.clone());
        let blit_texture_pipeline = draw_texture::pipeline(
            vk_ctx.device.clone(),
            final_render_pass.render_pass.clone(),
            vec![blit_texture_layout.clone()],
        );
        let sampler = Sampler::new(
            vk_ctx.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();
        let blit_desc = draw_texture::set_desc(
            blit_texture_layout.clone(),
            vk_ctx.set_alloc.clone(),
            sampler,
            forward_images[0].clone(),
        );

        Self {
            window,
            swapchain,
            images,
            recreate_swapchain: false,
            previous_frame_end: Some(vulkano::sync::now(vk_ctx.device.clone()).boxed()),

            forward_images,
            forward_render_pass,
            forward_viewport,
            final_render_pass,
            final_viewport,
            renderer,

            blit_texture_layout,
            blit_texture_pipeline,
            blit_desc,
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
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;
            self.images = new_images;

            self.final_viewport.extent = [window_size.width as f32, window_size.height as f32];
            self.forward_render_pass.resize(self.forward_images.clone());
            self.final_render_pass.resize(&self.images, &[]);

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
        self.forward_render_pass
            .begin_render_pass(&mut builder, self.forward_viewport.clone());

        self.renderer.render_forward_render_pass(&mut builder);

        builder.end_render_pass(Default::default()).unwrap();

        // ===================================================================== Final Pass
        self.final_render_pass.begin_render_pass(
            &mut builder,
            image_index as usize,
            self.final_viewport.clone(),
        );
        builder
            .bind_pipeline_graphics(self.blit_texture_pipeline.clone())
            .unwrap();
        bind_descriptor_set(
            &mut builder,
            self.blit_texture_pipeline.layout().clone(),
            self.blit_desc.clone(),
        );

        self.renderer.fill_screen.draw(&mut builder);

        builder.end_render_pass(Default::default()).unwrap();

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

fn bind_descriptor_set(
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    pipeline_layout: Arc<PipelineLayout>,
    descriptor_set: Arc<DescriptorSet>,
) {
    builder
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline_layout,
            0,
            descriptor_set,
        )
        .unwrap();
}
