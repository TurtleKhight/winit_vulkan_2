use std::sync::Arc;

use nalgebra::Matrix4;
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
    shader::ShaderStages,
};

use crate::game::camera::Camera;

#[derive(Clone, Copy, BufferContents)]
#[repr(C)]
pub struct CameraUniform {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}
impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        let view = camera.calc_v_mtx().into();
        let proj = camera.calc_p_mtx();
        Self { view, proj }
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

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
        }
    }
}
