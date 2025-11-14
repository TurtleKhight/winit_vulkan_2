pub mod final_single_pass;
pub mod single_pass;

use std::sync::Arc;

use vulkano::{
    device::Device,
    format::Format,
    render_pass::{AttachmentLoadOp, AttachmentStoreOp, RenderPass},
};

struct ColourAttachment {
    format: Format,
    samples: u32,
    load_op: AttachmentLoadOp,
    store_op: AttachmentStoreOp,
}

fn single_pass_renderpass(
    device: Arc<Device>,
    colour_attachments: &[ColourAttachment],
    colour: &[usize],
    depth_stencil: Option<usize>,
) -> Arc<RenderPass> {
    let create_info = {
        let attachment_num = colour_attachments.len() as u32;
        #[derive(Clone, Copy, Default)]
        struct Layouts {
            initial_layout: Option<vulkano::image::ImageLayout>,
            final_layout: Option<vulkano::image::ImageLayout>,
        }
        let mut layouts: Vec<Layouts> = vec![Layouts::default(); attachment_num as usize];
        let subpasses = vec![vulkano::render_pass::SubpassDescription {
            color_attachments: colour
                .iter()
                .map(|i| {
                    let layouts = &mut layouts[*i];
                    layouts.initial_layout = layouts
                        .initial_layout
                        .or(Some(vulkano::image::ImageLayout::ColorAttachmentOptimal));
                    layouts.final_layout =
                        Some(vulkano::image::ImageLayout::ColorAttachmentOptimal);
                    Some(vulkano::render_pass::AttachmentReference {
                        attachment: *i as u32,
                        layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
                        ..Default::default()
                    })
                })
                .collect(),
            color_resolve_attachments: vec![],
            depth_stencil_attachment: {
                depth_stencil.map(|depth_stencil| {
                    let layouts = &mut layouts[depth_stencil];
                    layouts.final_layout =
                        Some(vulkano::image::ImageLayout::DepthStencilAttachmentOptimal);
                    layouts.initial_layout = layouts.initial_layout.or(layouts.final_layout);
                    vulkano::render_pass::AttachmentReference {
                        attachment: depth_stencil as u32,
                        layout: vulkano::image::ImageLayout::DepthStencilAttachmentOptimal,
                        ..Default::default()
                    }
                })
            },
            depth_stencil_resolve_attachment: { None },
            depth_resolve_mode: { None },
            stencil_resolve_mode: { None },
            input_attachments: vec![],
            preserve_attachments: vec![],
            ..Default::default()
        }];
        let dependencies: Vec<_> = (0..subpasses.len().saturating_sub(1) as u32)
            .map(|id| {
                let src_stages = vulkano::sync::PipelineStages::ALL_GRAPHICS;
                let dst_stages = vulkano::sync::PipelineStages::ALL_GRAPHICS;
                let src_access = vulkano::sync::AccessFlags::MEMORY_READ
                    | vulkano::sync::AccessFlags::MEMORY_WRITE;
                let dst_access = vulkano::sync::AccessFlags::MEMORY_READ
                    | vulkano::sync::AccessFlags::MEMORY_WRITE;
                vulkano::render_pass::SubpassDependency {
                    src_subpass: id.into(),
                    dst_subpass: (id + 1).into(),
                    src_stages,
                    dst_stages,
                    src_access,
                    dst_access,
                    dependency_flags: vulkano::sync::DependencyFlags::BY_REGION,
                    ..Default::default()
                }
            })
            .collect();
        let attachments = (0..attachment_num)
            .map(|i| {
                let layouts = &mut layouts[i as usize];
                vulkano::render_pass::AttachmentDescription {
                    format: colour_attachments[i as usize].format,
                    samples: vulkano::image::SampleCount::try_from(
                        colour_attachments[i as usize].samples,
                    )
                    .unwrap(),
                    load_op: colour_attachments[i as usize].load_op,
                    store_op: colour_attachments[i as usize].store_op,
                    initial_layout: layouts.initial_layout.unwrap(),
                    final_layout: layouts.final_layout.unwrap(),
                    ..Default::default()
                }
            })
            .collect();

        vulkano::render_pass::RenderPassCreateInfo {
            attachments,
            subpasses,
            dependencies,
            ..Default::default()
        }
    };

    RenderPass::new(device, create_info).unwrap()
}
