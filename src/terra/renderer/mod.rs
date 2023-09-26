use super::{shader::loader::ShaderLoader, util};
use crate::{
    camera::Camera,
    sprite::{renderer::Color, SpriteData, SpriteRenderer},
    terra::vertex::Vertex,
};
use nalgebra_glm::{self as glm, Mat4};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RenderPassBeginInfo,
        SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, layout::DescriptorSetLayout,
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
};

#[derive(BufferContents)]
#[repr(C)]
pub struct GlobalData {
    projection: [f32; 16],
    view: [f32; 16],
}

impl GlobalData {
    pub fn new(projection: Mat4, view: Mat4) -> GlobalData {
        GlobalData {
            projection: mat4_to_array(projection),
            view: mat4_to_array(view),
        }
    }
}

#[derive(BufferContents)]
#[repr(C)]
pub struct PerObject {
    model: [f32; 16],
    color: [f32; 4],
}

impl PerObject {
    fn new(model: [f32; 16], color: &Color) -> PerObject {
        let color = color.get();

        PerObject {
            model,
            color: [color[0] as f32, color[1] as f32, color[2] as f32, 1.0],
        }
    }
}

pub struct SpriteResources {
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u32]>,
    image_set: Arc<PersistentDescriptorSet>,
}

pub struct GpuResources {
    sprite: HashMap<u64, SpriteResources>,

    global_buffer: Subbuffer<GlobalData>,
    global_descriptor_set: Arc<PersistentDescriptorSet>,
}

impl GpuResources {
    pub fn new(
        global_buffer: Subbuffer<GlobalData>,
        global_descriptor_set: Arc<PersistentDescriptorSet>,
    ) -> GpuResources {
        GpuResources {
            sprite: HashMap::new(),
            global_buffer,
            global_descriptor_set,
        }
    }
}

pub struct Renderer {
    sprites: HashMap<u64, Vec<Rc<RefCell<SpriteRenderer>>>>,
    camera: Option<Rc<RefCell<Camera>>>,
    pipeline: Arc<GraphicsPipeline>,
    allocator: StandardMemoryAllocator,
    command_pool: Arc<StandardCommandBufferAllocator>,
    descriptor_set_pool: Arc<StandardDescriptorSetAllocator>,
    resources: GpuResources,
    queue: Arc<Queue>,
    sampler: Arc<Sampler>,
    viewport: Viewport,
}

impl Renderer {
    pub fn new(
        device: &Arc<Device>,
        command_pool: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_pool: &Arc<StandardDescriptorSetAllocator>,
        render_pass: &Arc<RenderPass>,
        queue: &Arc<Queue>,
        shaders: &Rc<ShaderLoader>,
        viewport: Viewport,
    ) -> Renderer {
        let vs = shaders
            .vertex("sprite")
            .expect("Failed to find vertex shader");
        let fs = shaders
            .fragment("sprite")
            .expect("Failed to find fragment shader");

        let pipeline = util::create_graphics_pipeline(device, render_pass, &vs, &fs);
        let allocator = util::create_memory_pool(device);
        let global_buffer = create_global_buffer(&allocator);
        let global_descriptor_set = create_global_descriptor_set(
            descriptor_set_pool,
            &pipeline.layout().set_layouts()[0],
            &global_buffer,
        );

        let sampler = create_sampler(device);

        Renderer {
            sprites: HashMap::new(),
            camera: None,
            pipeline,
            viewport,
            sampler,
            allocator,
            command_pool: command_pool.clone(),
            descriptor_set_pool: descriptor_set_pool.clone(),
            queue: queue.clone(),
            resources: GpuResources::new(global_buffer, global_descriptor_set),
        }
    }

    pub fn add_sprite(&mut self, renderer: &Rc<RefCell<SpriteRenderer>>) {
        let _renderer = renderer.borrow();
        let sprite_id = _renderer.sprite().id();

        if let Some(sprites) = self.sprites.get_mut(&sprite_id) {
            sprites.push(renderer.clone());
        } else {
            let mut sprites = vec![];
            sprites.push(renderer.clone());
            self.sprites.insert(sprite_id, sprites);
        }

        println!("HERE");
        let sprite = _renderer.sprite();
        if let None = self.resources.sprite.get(&sprite_id) {
            let resources = create_sprite_resources(
                &self.allocator,
                &self.command_pool,
                &self.descriptor_set_pool,
                &self.pipeline.layout().set_layouts()[1],
                &self.sampler,
                &self.queue,
                sprite.load(),
            );

            self.resources.sprite.insert(sprite.id(), resources);
        }
    }

    pub fn set_camera(&mut self, camera: &Rc<RefCell<Camera>>) {
        self.camera = Some(camera.clone());
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub fn draw(&mut self, framebuffer: &Arc<Framebuffer>) -> Option<PrimaryAutoCommandBuffer> {
        if let Some(camera) = &self.camera {
            let mut builder = AutoCommandBufferBuilder::primary(
                &self.command_pool,
                self.queue.queue_family_index(),
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
                .set_viewport(0, [self.viewport.clone()])
                .bind_pipeline_graphics(self.pipeline.clone());

            let global_data = get_global_data(camera.clone());
            *self
                .resources
                .global_buffer
                .write()
                .expect("Failed to write to global buffer") = global_data;

            builder.bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                layout.clone(),
                0,
                [self.resources.global_descriptor_set.clone()]
                    .into_iter()
                    .collect::<Vec<_>>(),
            );

            for (id, renderers) in self.sprites.iter() {
                match self.resources.sprite.get(id) {
                    Some(resources) => {
                        builder
                            .bind_vertex_buffers(0, resources.vertex_buffer.clone())
                            .bind_index_buffer(resources.index_buffer.clone());

                        builder.bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            layout.clone(),
                            1,
                            [resources.image_set.clone()]
                                .into_iter()
                                .collect::<Vec<_>>(),
                        );

                        for renderer in renderers.iter() {
                            let model = mat4_to_array(renderer.borrow().transform().matrix());
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
        } else {
            None
        }
    }
}

fn mat4_to_array(matrix: Mat4) -> [f32; 16] {
    let mut mat = [0.0; 16];

    let matrix = matrix.data.as_slice();
    for index in 0..16 {
        mat[index] = matrix[index];
    }

    mat
}

fn create_global_buffer(allocator: &StandardMemoryAllocator) -> Subbuffer<GlobalData> {
    Buffer::from_data(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        GlobalData::new(Mat4::identity(), Mat4::identity()),
    )
    .expect("Failed to create Global Buffer")
}

fn get_global_data(camera: Rc<RefCell<Camera>>) -> GlobalData {
    let camera = camera.borrow();
    let clipping = camera.clipping_planes();
    let size = camera.size();
    let view = camera.transform().matrix();

    let ortho = glm::ortho(-size, size, -size, size, clipping.near, clipping.far);

    GlobalData::new(ortho, view)
}

fn create_sampler(device: &Arc<Device>) -> Arc<Sampler> {
    let create_info = SamplerCreateInfo {
        mag_filter: Filter::Linear,
        min_filter: Filter::Linear,
        address_mode: [SamplerAddressMode::Repeat; 3],
        ..Default::default()
    };

    Sampler::new(device.clone(), create_info).expect("Failed to create sampler.")
}

fn create_buffer<T: std::marker::Send + std::marker::Sync + BufferContents + Clone, I>(
    allocator: &StandardMemoryAllocator,
    usage: BufferUsage,
    iter: I,
) -> Subbuffer<[T]>
where
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    Buffer::from_iter(
        allocator,
        BufferCreateInfo {
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        iter,
    )
    .expect("Failed to create destination buffer ")
}

fn create_sprite_resources(
    allocator: &StandardMemoryAllocator,
    command_pool: &Arc<StandardCommandBufferAllocator>,
    descriptor_set_pool: &StandardDescriptorSetAllocator,
    layout: &Arc<DescriptorSetLayout>,
    sampler: &Arc<Sampler>,
    queue: &Arc<Queue>,
    data: SpriteData,
) -> SpriteResources {
    let vertex_buffer = create_buffer(allocator, BufferUsage::VERTEX_BUFFER, data.vertices);
    let index_buffer = create_buffer(allocator, BufferUsage::INDEX_BUFFER, data.indices);
    let image_set = create_image_set(
        allocator,
        command_pool,
        descriptor_set_pool,
        layout,
        sampler,
        queue,
        data,
    );

    SpriteResources {
        vertex_buffer,
        index_buffer,
        image_set,
    }
}

fn create_image_set(
    allocator: &StandardMemoryAllocator,
    command_pool: &Arc<StandardCommandBufferAllocator>,
    descriptor_set_pool: &StandardDescriptorSetAllocator,
    layout: &Arc<DescriptorSetLayout>,
    sampler: &Arc<Sampler>,
    queue: &Arc<Queue>,
    data: SpriteData,
) -> Arc<PersistentDescriptorSet> {
    let mut builder = AutoCommandBufferBuilder::primary(
        command_pool,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let image = ImmutableImage::from_iter(
        allocator,
        data.pixels.into_iter(),
        ImageDimensions::Dim2d {
            width: data.width,
            height: data.height,
            array_layers: 1,
        },
        vulkano::image::MipmapsCount::One,
        Format::R8G8B8A8_SRGB,
        &mut builder,
    )
    .expect("Failed to create image for sprite");

    let image_view =
        ImageView::new_default(image).expect("Failed to create image view for sprite.");

    let command_buffer = builder
        .build()
        .expect("Failed to create image set command buffer");

    let _future = command_buffer
        .execute(queue.clone())
        .expect("Failed to execute command buffer.");

    create_image_descriptor_set(descriptor_set_pool, layout, &image_view, sampler)
}

fn create_global_descriptor_set(
    allocator: &StandardDescriptorSetAllocator,
    layout: &Arc<DescriptorSetLayout>,
    buffer: &Subbuffer<GlobalData>,
) -> Arc<PersistentDescriptorSet> {
    PersistentDescriptorSet::new(
        allocator,
        layout.clone(),
        [WriteDescriptorSet::buffer(0, buffer.clone())],
    )
    .expect("Failed to create global descriptor set")
}

fn create_image_descriptor_set(
    allocator: &StandardDescriptorSetAllocator,
    layout: &Arc<DescriptorSetLayout>,
    image: &Arc<ImageView<ImmutableImage>>,
    sampler: &Arc<Sampler>,
) -> Arc<PersistentDescriptorSet> {
    PersistentDescriptorSet::new(
        allocator,
        layout.clone(),
        [WriteDescriptorSet::image_view_sampler(
            0,
            image.clone(),
            sampler.clone(),
        )],
    )
    .expect("Failed to create image descriptor set")
}
