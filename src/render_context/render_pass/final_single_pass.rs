use std::sync::Arc;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    device::Device,
    format::{ClearValue, Format},
    image::{Image, view::ImageView},
    pipeline::graphics::viewport::Viewport,
    render_pass::{
        AttachmentLoadOp, AttachmentStoreOp, Framebuffer, FramebufferCreateInfo, RenderPass,
    },
};

use crate::render_context::render_pass::{ColourAttachment, single_pass_renderpass};

pub struct FinalSingleRenderPass {
    pub render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,

    clear_colour: Option<ClearValue>,
}
impl FinalSingleRenderPass {
    pub fn new(
        device: &Arc<Device>,
        images: &[Arc<Image>],
        image_format: Format,
        depth: Option<Arc<ImageView>>,
    ) -> Self {
        // let clear_colour = None;
        let clear_colour = Some(ClearValue::Float([1.0, 0.0, 1.0, 1.0]));
        let load_op = if clear_colour.is_none() {
            AttachmentLoadOp::Load
        } else {
            AttachmentLoadOp::Clear
        };
        let store_op = AttachmentStoreOp::DontCare;

        let render_pass = single_pass_renderpass(
            device.clone(),
            &[ColourAttachment {
                format: image_format,
                samples: 1,
                load_op,
                store_op,
            }],
            &[0],
            None,
        );

        let framebuffers = Self::create_framebuffers(images, &render_pass);
        Self {
            render_pass,
            framebuffers,
            clear_colour,
        }
    }

    fn create_framebuffers(
        images: &[Arc<Image>],
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        let framebuffers = images
            .iter()
            .map(|image| {
                let final_colour = ImageView::new_default(image.clone()).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![final_colour],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        framebuffers
    }

    pub fn resize(&mut self, new_images: &[Arc<Image>]) {
        self.framebuffers = Self::create_framebuffers(new_images, &self.render_pass);
    }

    pub fn begin_render_pass(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        image_index: usize,
        viewport: Viewport,
    ) {
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![self.clear_colour],

                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_index].clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(0, [viewport].into_iter().collect())
            .unwrap();
    }
}
