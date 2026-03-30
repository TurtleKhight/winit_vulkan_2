use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{DescriptorSet, layout::DescriptorSetLayout},
    device::Device,
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageUsage,
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView,
    },
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, graphics::viewport::Viewport,
    },
    render_pass::RenderPass,
};

use crate::{
    game::Game,
    gui::Gui,
    msgln,
    render_context::renderer::{
        camera::CameraUniform,
        fill_screen::FillScreen,
        render_pass::{final_single_pass::FinalSingleRenderPass, single_pass::SingleRenderPass},
        sky::SkyUniform,
    },
    vulkan::VulkanContext,
};

mod camera;
mod draw_texture;
mod fill_screen;
mod render_pass;
mod sky;

pub struct Renderer {
    render_passes: RenderPasses,
    layouts: Layouts,
    pipelines: Pipelines,

    camera_set: Arc<DescriptorSet>,
    camera_buffer: Subbuffer<CameraUniform>,

    sky_buffer: Subbuffer<SkyUniform>,
    sky_set: Arc<DescriptorSet>,

    blit_desc: Arc<DescriptorSet>,
    fill_screen: FillScreen,
}
impl Renderer {
    pub fn new(
        vk_ctx: &VulkanContext,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        swap_images: &[Arc<Image>],
        swap_format: Format,
        extent: [u32; 3],
    ) -> Self {
        let render_passes = RenderPasses::new(vk_ctx, swap_images, swap_format, extent);
        let layouts = Layouts::new(vk_ctx.device.clone());
        let pipelines = Pipelines::new(vk_ctx.device.clone(), &layouts, &render_passes);

        let fill_screen = FillScreen::triangle(vk_ctx.mem_alloc.clone());

        let (camera_buffer, camera_set) = CameraUniform::default().set_desc(
            layouts.camera.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        let (sky_buffer, sky_set) = SkyUniform::default().set_desc(
            layouts.sky.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        let blit_desc = draw_texture::set_desc(
            layouts.blit_texture.clone(),
            vk_ctx.set_alloc.clone(),
            render_passes.sampler.clone(),
            render_passes.forward_images[0].clone(),
        );

        Self {
            render_passes,
            layouts,
            pipelines,

            camera_set,
            camera_buffer,

            sky_buffer,
            sky_set,

            blit_desc,

            fill_screen,
        }
    }

    pub fn get_final_pass(&self) -> Arc<RenderPass> {
        return self.render_passes.final_pass.render_pass.clone();
    }

    pub fn extent(&self) -> [u32; 3] {
        self.render_passes.extent()
    }

    pub fn resize_swap(&mut self, images: &[Arc<Image>]) {
        self.render_passes.resize_swap(images);
    }

    pub fn get_gbuffers(&self) -> &[Arc<ImageView>] {
        &self.render_passes.forward_images
    }

    pub fn resize_buffers(&mut self, vk_ctx: &VulkanContext, extent: [u32; 3]) {
        self.render_passes
            .resize_buffers(vk_ctx.device.clone(), vk_ctx.mem_alloc.clone(), extent);
        // self.pipelines = Pipelines::new(vk_ctx.device.clone(), &self.layouts, &self.render_passes);

        self.blit_desc = draw_texture::set_desc(
            self.layouts.blit_texture.clone(),
            vk_ctx.set_alloc.clone(),
            self.render_passes.sampler.clone(),
            self.render_passes.forward_images[0].clone(),
        );
    }

    pub fn update(&mut self, dt: f32, game: &Game, vk_ctx: &VulkanContext) {
        (self.camera_buffer, self.camera_set) = CameraUniform::new(&game.camera).set_desc(
            self.layouts.camera.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        (self.sky_buffer, self.sky_set) = SkyUniform::new(&game.camera).set_desc(
            self.layouts.sky.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );
    }

    pub fn render_forward_render_pass(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        // ============================================================================== START
        self.render_passes
            .forward_pass
            .begin_render_pass(builder, self.render_passes.forward_viewport.clone());
        // ============================================================================== Sky
        let pipeline = self.pipelines.sky.clone();
        builder.bind_pipeline_graphics(pipeline.clone()).unwrap();
        bind_descriptor_set(builder, pipeline.layout().clone(), self.sky_set.clone());
        self.fill_screen.draw(builder);
        // ============================================================================== END
        builder.end_render_pass(Default::default()).unwrap();
    }

    pub fn final_render_pass(
        &self,
        gui: &mut Gui,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        image_index: usize,
        swapchain_viewport: Viewport,
    ) {
        // ============================================================================== START
        self.render_passes
            .final_pass
            .begin_render_pass(builder, image_index, swapchain_viewport);
        // ============================================================================== Blit forward to swap
        let pipeline = self.pipelines.blit_texture.clone();
        builder.bind_pipeline_graphics(pipeline.clone()).unwrap();
        bind_descriptor_set(builder, pipeline.layout().clone(), self.blit_desc.clone());
        self.fill_screen.draw(builder);
        // ============================================================================== Gui
        gui.render(builder);
        // ============================================================================== END
        builder.end_render_pass(Default::default()).unwrap();
    }
}

struct RenderPasses {
    final_pass: FinalSingleRenderPass,
    forward_images: Vec<Arc<ImageView>>,
    forward_viewport: Viewport,
    forward_pass: SingleRenderPass,
    sampler: Arc<Sampler>,
}
impl RenderPasses {
    fn new(
        vk_ctx: &VulkanContext,
        swap_images: &[Arc<Image>],
        swap_format: Format,
        extent: [u32; 3],
    ) -> Self {
        let forward_viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };
        let forward_images = Self::forward_images(vk_ctx.mem_alloc.clone(), extent);
        let forward_pass =
            SingleRenderPass::new(vk_ctx.device.clone(), forward_images.clone(), true);
        let final_pass =
            FinalSingleRenderPass::new(vk_ctx.device.clone(), swap_images, swap_format, &[], false);

        let filer = Filter::Nearest;
        let sampler = Sampler::new(
            vk_ctx.device.clone(),
            SamplerCreateInfo {
                mag_filter: filer,
                min_filter: filer,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();
        Self {
            final_pass,
            forward_pass,
            forward_images,
            forward_viewport,
            sampler,
        }
    }

    fn forward_images(
        mem_alloc: Arc<StandardMemoryAllocator>,
        extent: [u32; 3],
    ) -> Vec<Arc<ImageView>> {
        let depth = ImageView::new_default(
            Image::new(
                mem_alloc.clone(),
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
                mem_alloc,
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
        return forward_images;
    }

    fn resize_swap(&mut self, swap_images: &[Arc<Image>]) {
        self.final_pass.resize(swap_images, &[]);
    }

    fn extent(&self) -> [u32; 3] {
        self.forward_images[0].image().extent()
    }

    fn resize_buffers(
        &mut self,
        device: Arc<Device>,
        mem_alloc: Arc<StandardMemoryAllocator>,
        extent: [u32; 3],
    ) {
        msgln!("RESIZING TO: {:?}", extent);
        self.forward_viewport.extent = [extent[0] as f32, extent[1] as f32];
        self.forward_images = Self::forward_images(mem_alloc, extent);
        self.forward_pass =
            SingleRenderPass::new(device.clone(), self.forward_images.clone(), true);
    }
}

struct Layouts {
    camera: Arc<DescriptorSetLayout>,
    sky: Arc<DescriptorSetLayout>,
    blit_texture: Arc<DescriptorSetLayout>,
}
impl Layouts {
    fn new(device: Arc<Device>) -> Self {
        let camera = CameraUniform::set_layout(device.clone());
        let sky = SkyUniform::set_layout(device.clone());
        let blit_texture = draw_texture::set_layout(device);
        Self {
            camera,
            sky,
            blit_texture,
        }
    }
}

struct Pipelines {
    sky: Arc<GraphicsPipeline>,
    blit_texture: Arc<GraphicsPipeline>,
}
impl Pipelines {
    fn new(device: Arc<Device>, layouts: &Layouts, render_passes: &RenderPasses) -> Self {
        let sky = sky::pipeline(
            device.clone(),
            render_passes.forward_pass.render_pass.clone(),
            vec![layouts.sky.clone()],
        );
        let blit_texture = draw_texture::pipeline(
            device,
            render_passes.final_pass.render_pass.clone(),
            vec![layouts.blit_texture.clone()],
        );
        Self { sky, blit_texture }
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
