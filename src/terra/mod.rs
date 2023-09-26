pub mod renderer;
pub mod shader;
pub mod util;
pub mod vertex;
// Create Instance
// Create Surface
// Create Physical Device
// Create Logical Device
// Create Queues
// Create Command Pool
// Create Create Command Buffers
// Create Swapchain
// Create RenderTargets
// Create RenderPass
// Create FrameBuffers
// Create Pipelines??

use crate::{
    camera::Camera,
    sprite::{loader::SpriteLoader, SpriteRenderer},
};

use self::{renderer::Renderer, shader::loader::ShaderLoader};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    image::SwapchainImage,
    instance::Instance,
    render_pass::{Framebuffer, RenderPass},
    swapchain::{
        self, AcquireError, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{FlushError, GpuFuture},
    VulkanLibrary,
};
use winit::{event_loop::EventLoop, window::WindowId};

type AcquireImageResult = (u32, bool, SwapchainAcquireFuture);

pub struct Terra {
    library: Arc<VulkanLibrary>,
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    graphics: Arc<Queue>,
    command_pool: Arc<StandardCommandBufferAllocator>,
    descriptor_set_pool: Arc<StandardDescriptorSetAllocator>,
    swapchain: Arc<Swapchain>,
    render_targets: Vec<Arc<SwapchainImage>>,
    render_pass: Arc<RenderPass>,
    frame_buffers: Vec<Arc<Framebuffer>>,
    shaders: Rc<ShaderLoader>,
    sprites: Rc<SpriteLoader>,
    renderer: Renderer,
}

impl Terra {
    pub fn init(events: &EventLoop<()>) -> Terra {
        let library = util::create_library();
        let instance = util::create_instance(&library);
        let surface: Arc<Surface> = util::create_surface(&instance, events);
        let (device, graphics) = util::create_device(&instance, &surface);
        let command_pool = util::create_command_pool(&device);
        let descriptor_set_pool = util::create_descriptor_set_pool(&device);
        let (swapchain, render_targets) = util::create_swap_chain(&device, &surface);
        let render_pass = util::create_render_pass(&device, swapchain.image_format());
        let frame_buffers = util::create_frame_buffers(&render_targets, &render_pass);
        let viewport = util::create_viewport(util::get_surface_dimensions(&surface));
        let shaders = Rc::new(ShaderLoader::load(&device));
        let sprites = Rc::new(SpriteLoader::load());

        let renderer = Renderer::new(
            &device,
            &command_pool,
            &descriptor_set_pool,
            &render_pass,
            &graphics,
            &shaders,
            viewport,
        );

        Terra {
            library,
            instance,
            surface: surface.clone(),
            device,
            graphics,
            command_pool,
            descriptor_set_pool,
            swapchain,
            render_targets,
            render_pass,
            frame_buffers,
            shaders,
            sprites,
            renderer,
        }
    }
}

impl Terra {
    pub fn render(&mut self) {
        if let Some((image, mut suboptimal, acquire_future)) =
            self.acquire_swapchain_image(&self.swapchain)
        {
            let framebuffer = &self.frame_buffers[image as usize];

            if let Some(command_buffer) = self.renderer.draw(&framebuffer) {
                let future = acquire_future
                    .then_execute(self.graphics.clone(), command_buffer)
                    .expect("Failed to execute command buffer")
                    .then_swapchain_present(
                        self.graphics.clone(),
                        SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => future
                        .wait(None)
                        .expect("Fence failed to signal completion."),
                    Err(FlushError::OutOfDate) => suboptimal = true,
                    Err(FlushError::ResourceAccessError { error, use_ref }) => {
                        let resource = use_ref.unwrap().resource_in_command;
                        panic!("Access to resource denied {:?} {:?}", error, resource);
                    }
                    Err(e) => panic!("Failed to flush future: {e}"),
                }
            } else {
                let present_info =
                    SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image);
                let _ = acquire_future
                    .then_swapchain_present(self.graphics.clone(), present_info)
                    .then_signal_fence_and_flush()
                    .unwrap()
                    .wait(None);
            }

            if !suboptimal {
                return;
            }
        }

        self.recreate_swapchain();
    }

    pub fn window_id(&self) -> WindowId {
        util::get_surface_window(&self.surface).id()
    }

    pub fn sprites(&self) -> &Rc<SpriteLoader> {
        &self.sprites
    }

    pub fn set_camera(&mut self, camera: &Rc<RefCell<Camera>>) {
        self.renderer.set_camera(camera);
    }

    pub fn add_sprite_renderer(&mut self, sprite_renderer: &Rc<RefCell<SpriteRenderer>>) {
        self.renderer.add_sprite(sprite_renderer);
    }

    // Returns None if the swapchain needs to be recreated
    fn acquire_swapchain_image(&self, swapchain: &Arc<Swapchain>) -> Option<AcquireImageResult> {
        match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => Some(r),
            Err(AcquireError::OutOfDate) => None,
            Err(e) => panic!("Failed to acquire next swapchain image: {e}"),
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let create_info = SwapchainCreateInfo {
            image_extent: util::get_surface_dimensions(&self.surface),
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
        let viewport = util::create_viewport(util::get_surface_dimensions(&self.surface));
        self.renderer.set_viewport(viewport);
    }
}
