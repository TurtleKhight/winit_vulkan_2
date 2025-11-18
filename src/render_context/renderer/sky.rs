use std::sync::Arc;

use nalgebra::{Matrix4, Vector4};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
    },
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
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

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "./shaders/fill_screen_vs.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./shaders/sky_fs.glsl",

    }
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
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState {
                        write_enable: true,
                        compare_op: CompareOp::LessOrEqual,
                    }),
                    ..Default::default()
                }),
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

use crate::{game::camera::Camera, render_context::renderer::fill_screen::FillScreenVertex};

#[derive(Clone, Copy, BufferContents)]
#[repr(C)]
pub struct SkyUniform {
    pv_inv_mtx: Matrix4<f32>,
    sky_colour: Vector4<f32>,
    ground_colour: Vector4<f32>,
}
impl SkyUniform {
    pub fn new(camera: &Camera) -> Self {
        let p_mtx = camera.calc_p_mtx();
        let p_mtx = p_mtx.try_inverse().unwrap_or(p_mtx);
        let c_mtx = camera.calc_dir_mtx();
        let pv_inv_mtx = c_mtx.to_homogeneous() * p_mtx;
        let ground_colour = Vector4::new(0.3, 0.3, 0.3, 1.0);
        let sky_colour = Vector4::new(0.5, 0.8, 1.0, 1.0);

        Self {
            pv_inv_mtx,
            ground_colour,
            sky_colour,
        }
    }

    pub fn set_layout(device: Arc<Device>) -> Arc<DescriptorSetLayout> {
        DescriptorSetLayout::new(
            device,
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                    },
                )]
                .into(),
                ..Default::default()
            },
        )
        .unwrap()
    }

    pub fn set_desc(
        &self,
        layout: Arc<DescriptorSetLayout>,
        mem_alloc: Arc<StandardMemoryAllocator>,
        set_alloc: Arc<StandardDescriptorSetAllocator>,
    ) -> (Subbuffer<Self>, Arc<DescriptorSet>) {
        let buffer = Buffer::from_data(
            mem_alloc,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            *self,
        )
        .unwrap();
        let desc = DescriptorSet::new(
            set_alloc,
            layout,
            [WriteDescriptorSet::buffer(0, buffer.clone())],
            [],
        )
        .unwrap();
        (buffer, desc)
    }
}

impl Default for SkyUniform {
    fn default() -> Self {
        Self {
            pv_inv_mtx: Matrix4::identity(),
            ground_colour: Vector4::zeros(),
            sky_colour: Vector4::zeros(),
        }
    }
}
