use std::sync::Arc;

use glam::Mat4;
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferContents, BufferUsage, Subbuffer,
    },
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
    },
    descriptor_set::{
        allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator},
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::{self, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::Swapchain,
    Validated, ValidationError, VulkanError,
};

use super::mvp::MVP;

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/renderer/vulkan/shaders/default_lit.vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/renderer/vulkan/shaders/default_lit.frag.glsl",
    }
}

#[derive(BufferContents, Vertex, Debug, Clone, Copy)]
#[repr(C)]
pub struct MeshVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

#[derive(BufferContents)]
#[repr(C)]
pub struct DefaultLitIndex {
    pub index: u32,
}

pub struct DefaultLitPipeline {
    layout: Arc<PipelineLayout>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
}

impl DefaultLitPipeline {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
        swapchain_images: &[Arc<Image>],
        extent: [f32; 2],
    ) -> Result<Self, Validated<VulkanError>> {
        let vs = vs::load(device.clone())?.entry_point("main").unwrap();
        let fs = fs::load(device.clone())?.entry_point("main").unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: extent,
            depth_range: 0.0..=1.0,
        };

        let vertex_input_state = MeshVertex::per_vertex().definition(&vs.info().input_interface)?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs.clone()),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|err| err.error)?,
        )?;

        let render_pass = DefaultLitPipeline::create_render_pass(device, swapchain)?;
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let graphics_pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state.clone()),
                input_assembly_state: Some(InputAssemblyState {
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState {
                    viewports: [viewport.clone()].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState {
                    polygon_mode: PolygonMode::Fill,
                    cull_mode: CullMode::Back,
                    front_face: FrontFace::Clockwise,
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                subpass: Some(subpass.clone().into()),
                ..GraphicsPipelineCreateInfo::layout(layout.clone())
            },
        )?;
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let framebuffers = DefaultLitPipeline::create_framebuffers(
            &memory_allocator,
            &swapchain_images,
            &render_pass,
        )
        .unwrap();

        Ok(DefaultLitPipeline {
            layout,
            graphics_pipeline,
            framebuffers,
        })
    }

    pub fn create_command_buffers(
        &self,
        command_buffer_allocator: &StandardCommandBufferAllocator,
        queue: Arc<Queue>,
        verticies: &Subbuffer<[MeshVertex]>,
        indicies: &Subbuffer<[u32]>,
        descriptor_set: Arc<PersistentDescriptorSet>,
    ) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, Validated<VulkanError>> {
        self.framebuffers
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )?;

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.0, 0.0, 0.0, 1.0].into()),
                                Some(format::ClearValue::Depth(1.0)),
                            ],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .bind_pipeline_graphics(self.graphics_pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, verticies.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        vulkano::pipeline::PipelineBindPoint::Graphics,
                        self.layout.clone(),
                        0,
                        descriptor_set.clone(),
                    )
                    .unwrap()
                    .bind_index_buffer(indicies.clone())
                    .unwrap()
                    .draw_indexed(indicies.len() as u32, 1, 0, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();

                builder.build()
            })
            .collect()
    }

    fn create_render_pass(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> Result<Arc<RenderPass>, Validated<vulkano::VulkanError>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
            },
        )
    }

    fn create_framebuffers(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        images: &[Arc<Image>],
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>, Validated<vulkano::VulkanError>> {
        let extent = images[0].extent();
        let depth_stencil_image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                format: Format::D16_UNORM,
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                extent: extent,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )
        .unwrap();
        let depth_stencil_view = ImageView::new_default(depth_stencil_image.clone()).unwrap();

        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone())?;

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_stencil_view.clone()],
                        ..Default::default()
                    },
                )
            })
            .collect()
    }

    pub fn create_descriptor_set(
        &self,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        mvp: MVP,
    ) -> Result<Arc<PersistentDescriptorSet>, Validated<VulkanError>> {
        let mvp_buffer = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let mvp_buffer_subbuffer = {
            let clip_space = mvp.to_clip_space();

            let mvp_data = vs::MVP_Data {
                clip_space: clip_space.to_cols_array_2d(),
            };

            let subbuffer = mvp_buffer.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = mvp_data;

            subbuffer
        };

        let descriptor_set_layout = self.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            descriptor_set_allocator,
            descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, mvp_buffer_subbuffer)],
            [],
        )
    }

    pub fn layout(&self) -> &PipelineLayout {
        self.layout.as_ref()
    }
}
