use std::sync::Arc;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    device::Device,
    format::{ClearValue, Format},
    image::view::ImageView,
    pipeline::graphics::viewport::Viewport,
    render_pass::{
        AttachmentLoadOp, AttachmentStoreOp, Framebuffer, FramebufferCreateInfo, RenderPass,
    },
};

use super::ColourAttachment;
use super::single_pass_renderpass;

pub struct SingleRenderPass {
    pub render_pass: Arc<RenderPass>,
    framebuffer: Arc<Framebuffer>,
    clear_values: Vec<Option<ClearValue>>,
}
impl SingleRenderPass {
    pub fn new(device: &Arc<Device>, images: Vec<Arc<ImageView>>, is_last_depth: bool) -> Self {
        let n = images.len();
        let mut colour_attachments = Vec::with_capacity(n);
        let mut clear_values = Vec::with_capacity(n);
        for image in &images {
            let load_op = if image.format() == Format::D16_UNORM {
                AttachmentLoadOp::Clear
            } else {
                AttachmentLoadOp::Load
            };
            // let load_op = AttachmentLoadOp::Clear;
            let store_op = AttachmentStoreOp::Store;
            let attachment = ColourAttachment {
                format: image.format(),
                samples: 1,
                load_op,
                store_op,
            };
            colour_attachments.push(attachment);
            let clearvalue = Self::clear_value(image.format(), load_op);
            clear_values.push(clearvalue);
        }

        let (colour, depth_attachment) = if is_last_depth {
            ((0..n - 1).collect::<Vec<_>>(), Some(n - 1))
        } else {
            ((0..n).collect::<Vec<_>>(), None)
        };

        let render_pass = single_pass_renderpass(
            device.clone(),
            &colour_attachments,
            &colour,
            depth_attachment,
        );

        let framebuffer = Self::create_framebuffer(images, &render_pass);
        Self {
            render_pass,
            framebuffer,
            clear_values,
        }
    }

    fn clear_value(format: Format, load_op: AttachmentLoadOp) -> Option<ClearValue> {
        if load_op == AttachmentLoadOp::Clear {
            match format {
                Format::R8G8B8A8_UNORM | Format::R8G8B8A8_SRGB => {
                    return Some(ClearValue::Float([1.0, 0.0, 0.0, 1.0]));
                }
                Format::D16_UNORM => return Some(ClearValue::Depth(1.0)),
                _ => panic!("why am i using this format??? {:?}", format),
            }
        }
        return None;
    }

    fn create_framebuffer(
        attachments: Vec<Arc<ImageView>>,
        render_pass: &Arc<RenderPass>,
    ) -> Arc<Framebuffer> {
        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments,
                ..Default::default()
            },
        )
        .unwrap();
        framebuffer
    }

    pub fn resize(&mut self, new_images: Vec<Arc<ImageView>>) {
        self.framebuffer = Self::create_framebuffer(new_images, &self.render_pass);
    }

    pub fn begin_render_pass(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        viewport: Viewport,
    ) {
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: self.clear_values.clone(),
                    ..RenderPassBeginInfo::framebuffer(self.framebuffer.clone())
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
