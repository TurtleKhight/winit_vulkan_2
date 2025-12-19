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

use super::ColourAttachment;
use super::single_pass_renderpass;

pub struct FinalSingleRenderPass {
    pub render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    clear_values: Vec<Option<ClearValue>>,
}
impl FinalSingleRenderPass {
    pub fn new(
        device: Arc<Device>,
        swap_images: &[Arc<Image>],
        swap_image_format: Format,
        images: &[Arc<ImageView>],
        is_last_depth: bool,
    ) -> Self {
        let n = images.len();
        let mut colour_attachments = Vec::with_capacity(n);
        let mut clear_values = Vec::with_capacity(n + 1);
        // Swapchain is first the first colour attachment
        let load_op = AttachmentLoadOp::Load;
        let store_op = AttachmentStoreOp::Store;
        colour_attachments.push(ColourAttachment {
            format: swap_image_format,
            samples: 1,
            load_op,
            store_op,
        });
        clear_values.push(None);
        // Other attachments are next, depth being last
        for image in images {
            let attachment = ColourAttachment {
                format: image.format(),
                samples: 1,
                load_op,
                store_op,
            };
            colour_attachments.push(attachment);
            let clearvalue = None;
            clear_values.push(clearvalue);
        }
        let (colour, depth_attachment) = if is_last_depth {
            ((0..n).collect::<Vec<_>>(), Some(n))
        } else {
            ((0..=n).collect::<Vec<_>>(), None)
        };

        let render_pass =
            single_pass_renderpass(device, &colour_attachments, &colour, depth_attachment);

        let framebuffers = Self::create_framebuffers(swap_images, &images, render_pass.clone());
        Self {
            render_pass,
            framebuffers,
            clear_values,
        }
    }

    fn create_framebuffers(
        swap_images: &[Arc<Image>],
        images: &[Arc<ImageView>],
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        let framebuffers = swap_images
            .iter()
            .map(|image| {
                let final_colour = ImageView::new_default(image.clone()).unwrap();
                let mut attachments = Vec::with_capacity(images.len() + 1);
                attachments.push(final_colour);
                for image in images {
                    attachments.push(image.clone());
                }
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments,
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        framebuffers
    }

    pub fn resize(&mut self, swap_images: &[Arc<Image>], images: &[Arc<ImageView>]) {
        self.framebuffers =
            Self::create_framebuffers(swap_images, images, self.render_pass.clone());
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
                    clear_values: self.clear_values.clone(),
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
