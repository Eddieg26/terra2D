pub mod data;

use crate::terra::{
    context::GraphicsContext,
    data::{GlobalData, Vertex},
    programs::sprite::data::PerObject,
    resources::{gpu::GpuResources, graphics::GraphicsResources},
    util,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::PersistentDescriptorSet,
    image::{view::ImageView, ImmutableImage},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState, vertex_input::Vertex as BaseVertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::Framebuffer,
};

pub struct SpriteRenderProgram {
    context: Rc<RefCell<GraphicsContext>>,
    graphics_resources: Rc<RefCell<GraphicsResources>>,
    gpu_resources: Rc<RefCell<GpuResources>>,
    pipeline: Arc<GraphicsPipeline>,
    global_descriptor_set: Arc<PersistentDescriptorSet>,
    sprite_descriptor_sets: HashMap<u64, Arc<PersistentDescriptorSet>>,
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

        SpriteRenderProgram {
            context: context.clone(),
            graphics_resources: graphics_resources.clone(),
            gpu_resources: gpu_resources.clone(),
            pipeline,
            global_descriptor_set,
            sprite_descriptor_sets: HashMap::new(),
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

        self.update_global_buffer();

        builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            layout.clone(),
            0,
            [self.global_descriptor_set.clone()]
                .into_iter()
                .collect::<Vec<_>>(),
        );

        let graphics_resources = self.graphics_resources.clone();
        let graphics_resources = graphics_resources.borrow();
        let index_buffer = graphics_resources.sprite_index_buffer();

        let context = self.context.clone();
        let context = context.borrow();

        for (id, renderers) in context.sprite_renderers() {
            match graphics_resources.sprite(id) {
                Some(resources) => {
                    builder
                        .bind_vertex_buffers(0, resources.vertex_buffer().clone())
                        .bind_index_buffer(index_buffer.clone());

                    let set = self.get_or_create_image_set(id, resources.image());
                    builder.bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        layout.clone(),
                        1,
                        [set].into_iter().collect::<Vec<_>>(),
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

    fn update_global_buffer(&self) {
        let context = self.context.borrow();
        let camera = context.camera().borrow();

        let ortho = camera.ortho();
        let view = camera.transform().matrix();
        let global_data = GlobalData::new(ortho, view);

        *self
            .gpu_resources
            .borrow()
            .global_buffer()
            .write()
            .expect("Failed to write to global buffer") = global_data;
    }

    fn get_or_create_image_set(
        &mut self,
        id: &u64,
        image: &Arc<ImageView<ImmutableImage>>,
    ) -> Arc<PersistentDescriptorSet> {
        if let Some(set) = self.sprite_descriptor_sets.get(id) {
            set.clone()
        } else {
            let resources = self.gpu_resources.borrow();
            let allocator = resources.descriptor_set_alloc();
            let layout = &self.pipeline.layout().set_layouts()[1];
            let sampler = resources.sampler();

            let set = util::create_image_descriptor_set(allocator, layout, image, sampler);
            let clone = set.clone();
            self.sprite_descriptor_sets.insert(*id, set);
            clone
        }
    }
}

fn create_pipeline(resources: &Rc<RefCell<GpuResources>>) -> Arc<GraphicsPipeline> {
    let resources = resources.borrow();
    let device = resources.device();
    let render_pass = resources.render_pass();
    let shaders = resources.shaders();

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
