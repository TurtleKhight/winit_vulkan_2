use imgui::internal::RawWrapper;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::sync::Arc;
use vulkano::{
    DeviceSize,
    buffer::{
        Buffer, BufferContents, BufferCreateInfo, BufferUsage,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::{AutoCommandBufferBuilder, CopyBufferToImageInfo},
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator},
        layout::DescriptorSetLayout,
    },
    device::Device,
    format::Format,
    image::{
        Image, ImageAspects, ImageCreateInfo, ImageType, ImageUsage,
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView,
    },
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator,
    },
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
            },
            depth_stencil::{DepthState, DepthStencilState, StencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::Subpass,
};
use winit::{
    error::ExternalError,
    event::{Event, WindowEvent},
    window::Window,
};

pub struct Gui {
    pub ctx: imgui::Context,
    pub platform: WinitPlatform,
    pub window: Arc<Window>,
    pub renderer: Option<Renderer>,
}
impl Gui {
    pub fn new(window: Arc<Window>) -> Self {
        let mut ctx = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut ctx);
        platform.attach_window(ctx.io_mut(), &window, HiDpiMode::Default);
        ctx.set_platform_name("Winit".to_owned());
        ctx.set_renderer_name("Vulkano".to_owned());

        Self {
            ctx,
            platform,
            renderer: None,
            window,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) {
        self.platform.handle_event(
            self.ctx.io_mut(),
            &self.window,
            &Event::WindowEvent::<()> {
                window_id: self.window.id(),
                event: event.clone(),
            },
        );
    }

    pub fn ui(&mut self, content: impl FnOnce(&mut imgui::Ui)) -> Result<(), ExternalError> {
        self.platform
            .prepare_frame(self.ctx.io_mut(), &self.window)?;
        let ui = self.ctx.new_frame();
        content(ui);
        self.platform.prepare_render(ui, &self.window);
        Ok(())
    }

    pub fn new_renderer(
        &mut self,
        device: Arc<Device>,
        mem_allocator: Arc<StandardMemoryAllocator>,
        set_allocator: Arc<StandardDescriptorSetAllocator>,
        subpass: Subpass,
        window: Arc<Window>,
    ) {
        self.window = window;
        self.renderer = Some(Renderer::new(device, mem_allocator, set_allocator, subpass));
    }

    pub fn render<L>(&mut self, builder: &mut AutoCommandBufferBuilder<L>) {
        let draw_data = self.ctx.render();
        if let Some(renderer) = &mut self.renderer {
            renderer.render(builder, draw_data);
        }
    }
}

pub struct Texture {
    image: Arc<Image>,
    set: Arc<DescriptorSet>,
}
impl Texture {
    pub fn new(
        mem_allocator: Arc<dyn MemoryAllocator>,
        set_allocator: Arc<dyn DescriptorSetAllocator>,
        layout: Arc<DescriptorSetLayout>,
        sampler: Arc<Sampler>,
        format: Format,
        width: u32,
        height: u32,
    ) -> Self {
        let image = Image::new(
            mem_allocator,
            ImageCreateInfo {
                format,
                extent: [width, height, 1],
                image_type: ImageType::Dim2d,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();
        let view = ImageView::new_default(image.clone()).unwrap();

        let set = DescriptorSet::new(
            set_allocator,
            layout,
            [WriteDescriptorSet::image_view_sampler(
                0,
                view.clone(),
                sampler,
            )],
            [],
        )
        .unwrap();

        Self { image, set }
    }
    pub fn write<L>(
        &self,
        allocator: Arc<dyn MemoryAllocator>,
        data: &[u8],
        builder: &mut AutoCommandBufferBuilder<L>,
    ) {
        let stage = Buffer::from_iter(
            allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data.iter().copied(),
        )
        .unwrap();

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                stage,
                self.image.clone(),
            ))
            .unwrap();
    }
}

pub struct Renderer {
    device: Arc<Device>,
    mem_allocator: Arc<StandardMemoryAllocator>,
    set_allocator: Arc<StandardDescriptorSetAllocator>,

    vertex_buffers: SubbufferAllocator,
    index_buffers: SubbufferAllocator,
    textures: imgui::Textures<Texture>,
    sampler: Arc<Sampler>,

    pipeline: Arc<GraphicsPipeline>,
}
impl Renderer {
    pub fn new(
        device: Arc<Device>,
        mem_allocator: Arc<StandardMemoryAllocator>,
        set_allocator: Arc<StandardDescriptorSetAllocator>,
        subpass: Subpass,
    ) -> Self {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = [ImguiVertex::per_vertex()].definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let (has_depth_buffer, has_stencil_buffer) = subpass
            .subpass_desc()
            .depth_stencil_attachment
            .as_ref()
            .map(|depth_stencil_attachment| {
                let aspects = subpass.render_pass().attachments()
                    [depth_stencil_attachment.attachment as usize]
                    .format
                    .aspects();
                (
                    aspects.intersects(ImageAspects::DEPTH),
                    aspects.intersects(ImageAspects::STENCIL),
                )
            })
            .unwrap_or((false, false));
        let depth_stencil_state = if has_depth_buffer || has_stencil_buffer {
            let depth = if has_depth_buffer {
                Some(DepthState::default())
            } else {
                None
            };
            let stencil = if has_stencil_buffer {
                Some(StencilState::default())
            } else {
                None
            };
            Some(DepthStencilState {
                depth,
                stencil,
                ..Default::default()
            })
        } else {
            None
        };

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                multisample_state: Some(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState {
                    front_face: FrontFace::Clockwise,
                    cull_mode: CullMode::None,
                    ..Default::default()
                }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend {
                            src_color_blend_factor: BlendFactor::SrcAlpha,
                            dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                            color_blend_op: BlendOp::Add,
                            src_alpha_blend_factor: BlendFactor::OneMinusDstAlpha,
                            dst_alpha_blend_factor: BlendFactor::One,
                            alpha_blend_op: BlendOp::Add,
                        }),
                        ..Default::default()
                    },
                )),
                depth_stencil_state,
                dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
                    .into_iter()
                    .collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();

        let textures = imgui::Textures::new();

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        let vertex_buffer = SubbufferAllocator::new(
            mem_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::VERTEX_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );
        let index_buffer = SubbufferAllocator::new(
            mem_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::INDEX_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        Self {
            device,
            mem_allocator,
            set_allocator,
            vertex_buffers: vertex_buffer,
            index_buffers: index_buffer,
            textures,
            sampler,
            pipeline,
        }
    }

    fn render<L>(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<L>,
        draw_data: &imgui::DrawData,
    ) {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if fb_width <= 0.0 || fb_height <= 0.0 {
            return;
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .push_constants(
                self.pipeline.layout().clone(),
                0,
                vs::PushConstants {
                    matrix: [
                        [(2.0 / (right - left)), 0.0, 0.0, 0.0],
                        [0.0, (2.0 / (bottom - top)), 0.0, 0.0],
                        [0.0, 0.0, -1.0, 0.0],
                        [
                            (right + left) / (left - right),
                            (top + bottom) / (top - bottom),
                            0.0,
                            1.0,
                        ],
                    ],
                },
            )
            .unwrap();

        if draw_data.draw_lists_count() == 0 {
            return;
        }
        for draw_list in draw_data.draw_lists() {
            let vertices: &[ImguiVertex] = unsafe { draw_list.transmute_vtx_buffer() };
            let vbuf = self
                .vertex_buffers
                .allocate_slice(vertices.len() as DeviceSize)
                .unwrap();
            let ibuf = self
                .index_buffers
                .allocate_slice(draw_list.idx_buffer().len() as DeviceSize)
                .unwrap();
            vbuf.write().unwrap().copy_from_slice(vertices);
            ibuf.write()
                .unwrap()
                .copy_from_slice(draw_list.idx_buffer());

            builder
                .bind_vertex_buffers(0, vbuf)
                .unwrap()
                .bind_index_buffer(ibuf)
                .unwrap();

            let clip_off = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;
            for cmd in draw_list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements { count, cmd_params } => {
                        let clip_rect = [
                            (cmd_params.clip_rect[0] - clip_off[0]) * clip_scale[0],
                            (cmd_params.clip_rect[1] - clip_off[1]) * clip_scale[1],
                            (cmd_params.clip_rect[2] - clip_off[0]) * clip_scale[0],
                            (cmd_params.clip_rect[3] - clip_off[1]) * clip_scale[1],
                        ];

                        if clip_rect[0] < fb_width
                            && clip_rect[1] < fb_height
                            && clip_rect[2] >= 0.0
                            && clip_rect[3] >= 0.0
                        {
                            let texture = self.textures.get(cmd_params.texture_id).unwrap();

                            builder
                                .bind_descriptor_sets(
                                    PipelineBindPoint::Graphics,
                                    self.pipeline.layout().clone(),
                                    0,
                                    texture.set.clone(),
                                )
                                .unwrap()
                                .set_scissor(
                                    0,
                                    [Scissor {
                                        offset: [
                                            clip_rect[0].max(0.0) as u32,
                                            clip_rect[1].max(0.0) as u32,
                                        ],
                                        extent: [
                                            (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                                            (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                                        ],
                                    }]
                                    .into_iter()
                                    .collect(),
                                )
                                .unwrap();
                            unsafe {
                                builder
                                    .draw_indexed(
                                        count as u32,
                                        1,
                                        cmd_params.idx_offset as u32,
                                        0,
                                        0,
                                    )
                                    .unwrap()
                            };
                        }
                    }
                    imgui::DrawCmd::ResetRenderState => todo!(),
                    imgui::DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                }
            }
        }
    }

    pub fn reload_font_textures<L>(
        &mut self,
        ctx: &mut imgui::Context,
        builder: &mut AutoCommandBufferBuilder<L>,
    ) {
        let fonts = ctx.fonts();
        self.textures.remove(fonts.tex_id);

        let handle = fonts.build_rgba32_texture();
        let font_texture = Texture::new(
            self.mem_allocator.clone(),
            self.set_allocator.clone(),
            self.texture_layout().clone(),
            self.sampler.clone(),
            Format::R8G8B8A8_UNORM,
            handle.width,
            handle.height,
        );

        font_texture.write(self.mem_allocator.clone(), handle.data, builder);
        fonts.tex_id = self.textures.insert(font_texture);

        fonts.clear_tex_data();
    }

    fn texture_layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.pipeline.layout().set_layouts()[0]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, BufferContents, Vertex)]
pub struct ImguiVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
    #[format(R8G8B8A8_UNORM)]
    pub color: [u8; 4],
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r#"
#version 450
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(push_constant) uniform PushConstants {
    mat4 matrix;
} push_constants;
layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

void main() {
    f_uv = uv;
    f_color = color;
    gl_Position = push_constants.matrix * vec4(position.xy, 0, 1);
}
"#
    }
}
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
#version 450
layout(location = 0) in vec2 f_uv;
layout(location = 1) in vec4 f_color;

layout(push_constant) uniform PushConstants {
    mat4 matrix;
} push_constants;
layout(binding = 0, set = 0) uniform sampler2D tex;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = f_color * texture(tex, f_uv.st);
    out_color = f_color * texture(tex, f_uv.st);
}
"#
    }
}
