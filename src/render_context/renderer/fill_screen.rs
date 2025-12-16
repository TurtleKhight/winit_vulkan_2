use std::sync::Arc;

use nalgebra::Vector2;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::vertex_input::Vertex,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct FillScreenVertex {
    #[format(R32G32_SFLOAT)]
    position: Vector2<f32>,
}
pub struct FillScreen {
    pub vertex_buffer: Subbuffer<[FillScreenVertex]>,
}
impl FillScreen {
    pub fn triangle(mem_alloc: Arc<StandardMemoryAllocator>) -> Self {
        let v0 = FillScreenVertex {
            position: Vector2::new(-1.0, 3.0),
        };
        let v1 = FillScreenVertex {
            position: Vector2::new(-1.0, -1.0),
        };
        let v2 = FillScreenVertex {
            position: Vector2::new(3.0, -1.0),
        };
        let vertices = [v0, v1, v2];
        // let indices = [0, 1, 2];

        let vertex_buffer = Buffer::from_iter(
            mem_alloc,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();
        Self { vertex_buffer }
    }

    pub fn quad(mem_alloc: Arc<StandardMemoryAllocator>) -> FillScreen {
        let v0 = FillScreenVertex {
            position: Vector2::new(-1.0, 1.0),
        };
        let v1 = FillScreenVertex {
            position: Vector2::new(-1.0, -1.0),
        };
        let v2 = FillScreenVertex {
            position: Vector2::new(1.0, -1.0),
        };
        let v3 = FillScreenVertex {
            position: Vector2::new(1.0, 1.0),
        };
        let vertices = [v0, v1, v2, v3];
        // let indices = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = Buffer::from_iter(
            mem_alloc,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();
        Self { vertex_buffer }
    }

    pub fn draw(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) {
        builder
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap();

        unsafe { builder.draw(self.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();
    }
}
