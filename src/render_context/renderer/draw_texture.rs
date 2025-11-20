use std::sync::Arc;

use vulkano::{
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
    },
    device::Device,
    image::{sampler::Sampler, view::ImageView},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::ViewportState,
        },
        layout::PipelineLayoutCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderStages,
};

use crate::render_context::renderer::fill_screen::FillScreenVertex;

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "./shaders/fill_screen_vs.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./shaders/draw_texture_fs.glsl",

    }
}

pub fn set_desc(
    layout: Arc<DescriptorSetLayout>,
    set_alloc: Arc<StandardDescriptorSetAllocator>,
    sampler: Arc<Sampler>,
    texture: Arc<ImageView>,
) -> Arc<DescriptorSet> {
    DescriptorSet::new(
        set_alloc,
        layout,
        [
            WriteDescriptorSet::sampler(0, sampler),
            WriteDescriptorSet::image_view(1, texture),
        ],
        [],
    )
    .unwrap()
}

pub fn set_layout(device: Arc<Device>) -> Arc<DescriptorSetLayout> {
    DescriptorSetLayout::new(
        device,
        DescriptorSetLayoutCreateInfo {
            bindings: [
                (
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::Sampler)
                    },
                ),
                (
                    1,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::SampledImage)
                    },
                ),
            ]
            .into(),
            ..Default::default()
        },
    )
    .unwrap()
}

pub fn pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    set_layouts: Vec<Arc<DescriptorSetLayout>>,
) -> Arc<GraphicsPipeline> {
    let pipeline = {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = [FillScreenVertex::per_vertex()].definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts,
                push_constant_ranges: vec![],
                ..Default::default()
            },
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),

                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };
    return pipeline;
}
