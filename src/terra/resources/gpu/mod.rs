use crate::terra::{data::GlobalData, shader::loader::ShaderLoader, util};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, Subbuffer},
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, layout::DescriptorSetLayout,
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    image::SwapchainImage,
    instance::Instance,
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, RenderPass},
    sampler::Sampler,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError},
};

pub struct GpuResources {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    render_targets: Vec<Arc<SwapchainImage>>,
    command_buffer_alloc: Arc<StandardCommandBufferAllocator>,
    memory_alloc: StandardMemoryAllocator,
    descriptor_set_alloc: Arc<StandardDescriptorSetAllocator>,
    viewport: Viewport,
    render_pass: Arc<RenderPass>,
    frame_buffers: Vec<Arc<Framebuffer>>,
    shaders: ShaderLoader,
    global_buffer: Subbuffer<GlobalData>,
    sampler: Arc<Sampler>,
}

impl GpuResources {
    pub fn init(instance: &Arc<Instance>, surface: &Arc<Surface>) -> GpuResources {
        let (device, queue) = util::create_device(instance, surface);
        let (swapchain, render_targets) = util::create_swap_chain(&device, &surface);
        let command_buffer_alloc = util::create_command_pool(&device);
        let memory_alloc = util::create_memory_pool(&device);
        let descriptor_set_alloc = util::create_descriptor_set_pool(&device);
        let viewport = util::create_viewport(util::get_surface_dimensions(surface));
        let render_pass = util::create_render_pass(&device, swapchain.image_format());
        let frame_buffers = util::create_frame_buffers(&render_targets, &render_pass);
        let shaders = ShaderLoader::load(&device);
        let global_buffer = util::buffer_from_data(
            &memory_alloc,
            GlobalData::identity(),
            BufferUsage::UNIFORM_BUFFER,
            MemoryUsage::Upload,
        );
        let sampler = util::create_sampler(&device);

        GpuResources {
            device,
            queue,
            surface: surface.clone(),
            swapchain,
            render_targets,
            command_buffer_alloc,
            memory_alloc,
            descriptor_set_alloc,
            viewport,
            render_pass,
            shaders,
            global_buffer,
            sampler,
            frame_buffers,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn command_buffer_alloc(&self) -> &Arc<StandardCommandBufferAllocator> {
        &self.command_buffer_alloc
    }

    pub fn memory_alloc(&self) -> &StandardMemoryAllocator {
        &self.memory_alloc
    }

    pub fn descriptor_set_alloc(&self) -> &Arc<StandardDescriptorSetAllocator> {
        &self.descriptor_set_alloc
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }

    pub fn shaders(&self) -> &ShaderLoader {
        &self.shaders
    }

    pub fn global_buffer(&self) -> &Subbuffer<GlobalData> {
        &self.global_buffer
    }

    pub fn sampler(&self) -> &Arc<Sampler> {
        &self.sampler
    }

    pub fn swapchain(&self) -> &Arc<Swapchain> {
        &self.swapchain
    }

    pub fn render_targets(&self) -> &Vec<Arc<SwapchainImage>> {
        &self.render_targets
    }

    pub fn frame_buffers(&self) -> &Vec<Arc<Framebuffer>> {
        &self.frame_buffers
    }

    pub fn surface(&self) -> &Arc<Surface> {
        &self.surface
    }

    pub fn create_global_descriptor_set(
        &self,
        layout: &Arc<DescriptorSetLayout>,
    ) -> Arc<PersistentDescriptorSet> {
        PersistentDescriptorSet::new(
            &self.descriptor_set_alloc,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.global_buffer.clone())],
        )
        .expect("Failed to create global descriptor set")
    }
}

impl GpuResources {
    pub fn recreate_swapchain(&mut self) {
        let surface = &self.surface;
        let create_info = SwapchainCreateInfo {
            image_extent: util::get_surface_dimensions(surface),
            ..self.swapchain.create_info()
        };

        let (swapchain, render_targets) = match self.swapchain.recreate(create_info) {
            Ok(results) => results,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                (self.swapchain.clone(), self.render_targets.clone())
            }
            Err(e) => panic!("Failed to recreate the swapchain: {e}"),
        };

        self.swapchain = swapchain;
        self.render_targets = render_targets;
        self.frame_buffers = util::create_frame_buffers(&self.render_targets, &self.render_pass);
        self.viewport = util::create_viewport(util::get_surface_dimensions(surface));
    }
}
