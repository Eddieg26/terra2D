pub mod data;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use vulkano::{
    buffer::{BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::PersistentDescriptorSet,
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState, vertex_input::Vertex as BaseVertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::Framebuffer,
};

use crate::terra::{
    context::GraphicsContext,
    data::Vertex,
    programs::sprite::data::PerObject,
    resources::{gpu::GpuResources, graphics::GraphicsResources},
    util,
};

pub struct SpriteRenderProgram {
    context: Rc<RefCell<GraphicsContext>>,
    graphics_resources: Rc<RefCell<GraphicsResources>>,
    gpu_resources: Rc<RefCell<GpuResources>>,
    pipeline: Arc<GraphicsPipeline>,
    global_descriptor_set: Arc<PersistentDescriptorSet>,
    index_buffer: Subbuffer<[u32]>,
}

impl SpriteRenderProgram {
    pub fn new(
        context: &Rc<RefCell<GraphicsContext>>,
        graphics_resources: &Rc<RefCell<GraphicsResources>>,
        gpu_resources: &Rc<RefCell<GpuResources>>,
    ) -> SpriteRenderProgram {
        let pipeline = create_pipeline(gpu_resources);
        let layout = &pipeline.layout().set_layouts()[0];
        let global_descriptor_set = gpu_resources.borrow().create_global_descriptor_set(layout);
        let index_buffer = create_index_buffer(gpu_resources.borrow().memory_alloc());

        SpriteRenderProgram {
            context: context.clone(),
            graphics_resources: graphics_resources.clone(),
            gpu_resources: gpu_resources.clone(),
            pipeline,
            global_descriptor_set,
            index_buffer,
        }
    }

    pub fn draw(&mut self, framebuffer: &Arc<Framebuffer>) -> Option<PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.gpu_resources.borrow().command_buffer_alloc(),
            self.gpu_resources.borrow().queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to allocate command buffer.");

        let layout = self.pipeline.layout().clone();
        let clear_values = vec![Some([0.08, 0.08, 0.08, 1.0].into())];
        let render_pass_begin_info = RenderPassBeginInfo {
            clear_values,
            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
        };

        builder
            .begin_render_pass(render_pass_begin_info, SubpassContents::Inline)
            .expect("Failed to start render pass.");

        builder
            .set_viewport(0, [self.gpu_resources.borrow().viewport().clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        // TODO: Set Global buffer data to camera transform
        // let global_data = get_global_data(camera.clone());
        // *self
        //     .resources
        //     .global_buffer
        //     .write()
        //     .expect("Failed to write to global buffer") = global_data;

        builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            layout.clone(),
            0,
            [self.global_descriptor_set.clone()]
                .into_iter()
                .collect::<Vec<_>>(),
        );

        for (id, renderers) in self.context.borrow().sprite_renderers() {
            match self.graphics_resources.borrow().sprite(id) {
                Some(resources) => {
                    builder
                        .bind_vertex_buffers(0, resources.vertex_buffer().clone())
                        .bind_index_buffer(self.index_buffer.clone());

                    builder.bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        layout.clone(),
                        1,
                        [resources.set().clone()].into_iter().collect::<Vec<_>>(),
                    );

                    for renderer in renderers.iter() {
                        let model = util::mat4_to_array(renderer.borrow().transform().matrix());
                        let data = PerObject::new(model, renderer.borrow().color());
                        builder.push_constants(layout.clone(), 0, data);

                        let _ = builder.draw_indexed(6, 1, 0, 0, 0);
                    }
                }

                _ => continue,
            }
        }

        builder
            .end_render_pass()
            .expect("Failed to end render pass.");

        Some(builder.build().expect("Failed to build command buffer"))
    }
}

fn create_pipeline(resources: &Rc<RefCell<GpuResources>>) -> Arc<GraphicsPipeline> {
    let instance = resources.borrow();
    let device = instance.device();
    let render_pass = instance.render_pass();
    let shaders = instance.shaders();

    let vs = shaders.vertex("sprite").unwrap();
    let vs = util::get_shader_entry_point(vs);
    let fs = shaders.fragment("sprite").unwrap();
    let fs = util::get_shader_entry_point(fs);
    let subpass = render_pass.clone().first_subpass();

    GraphicsPipeline::start()
        .vertex_input_state(Vertex::per_vertex())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .render_pass(subpass)
        .vertex_shader(vs, ())
        .fragment_shader(fs, ())
        .input_assembly_state(InputAssemblyState::default())
        .build(device.clone())
        .expect("Failed to build graphics pipeline")
}

fn create_index_buffer(allocator: &StandardMemoryAllocator) -> Subbuffer<[u32]> {
    let indices = [0, 1, 2, 1, 0, 3];

    util::buffer_from_iter(
        allocator,
        indices.into_iter(),
        BufferUsage::INDEX_BUFFER,
        MemoryUsage::Upload,
    )
}
