use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{DescriptorSet, layout::DescriptorSetLayout},
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout},
    render_pass::RenderPass,
};

use crate::{
    game::Game,
    render_context::renderer::{camera::CameraUniform, fill_screen::FillScreen, sky::SkyUniform},
    vulkan::VulkanContext,
};

mod camera;
mod fill_screen;
mod sky;

pub struct Renderer {
    camera_layout: Arc<DescriptorSetLayout>,
    camera_set: Arc<DescriptorSet>,
    camera_buffer: Subbuffer<CameraUniform>,

    sky_layout: Arc<DescriptorSetLayout>,
    sky_pipeline: Arc<GraphicsPipeline>,
    sky_buffer: Subbuffer<SkyUniform>,
    sky_set: Arc<DescriptorSet>,

    fill_screen: FillScreen,
}
impl Renderer {
    pub fn new(vk_ctx: &VulkanContext, forward_render_pass: Arc<RenderPass>) -> Self {
        let camera_layout = CameraUniform::layout(vk_ctx.device.clone());
        let (camera_buffer, camera_set) = CameraUniform::default().descriptor(
            camera_layout.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        let sky_layout = SkyUniform::layout(vk_ctx.device.clone());
        let sky_pipeline = sky::pipeline(
            vk_ctx.device.clone(),
            forward_render_pass.clone(),
            vec![sky_layout.clone()],
        );
        let (sky_buffer, sky_set) = SkyUniform::default().descriptor(
            sky_layout.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        let fill_screen = FillScreen::triangle(vk_ctx.mem_alloc.clone());

        Self {
            camera_layout,
            camera_set,
            camera_buffer,

            sky_layout,
            sky_pipeline,
            sky_buffer,
            sky_set,

            fill_screen,
        }
    }

    pub fn update(&mut self, dt: f32, game: &Game, vk_ctx: &VulkanContext) {
        (self.camera_buffer, self.camera_set) = CameraUniform::new(&game.camera).descriptor(
            self.camera_layout.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );

        (self.sky_buffer, self.sky_set) = SkyUniform::new(&game.camera).descriptor(
            self.sky_layout.clone(),
            vk_ctx.mem_alloc.clone(),
            vk_ctx.set_alloc.clone(),
        );
    }

    pub fn render_forward_render_pass(
        &self,
        // render_pass: Arc<GraphicsPipeline>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        builder
            .bind_pipeline_graphics(self.sky_pipeline.clone())
            .unwrap();
        bind_descriptor_set(
            builder,
            self.sky_pipeline.layout().clone(),
            self.sky_set.clone(),
        );
        self.fill_screen.draw(builder);
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
