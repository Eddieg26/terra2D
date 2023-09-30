pub mod context;
pub mod data;
pub mod programs;
pub mod resources;
pub mod shader;
pub mod util;

use self::{
    context::GraphicsContext,
    programs::sprite::SpriteRenderProgram,
    resources::{gpu::GpuResources, graphics::GraphicsResources},
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use vulkano::{
    instance::Instance,
    swapchain::{self, AcquireError, SwapchainAcquireFuture, SwapchainPresentInfo},
    sync::{FlushError, GpuFuture},
};
use winit::{event_loop::EventLoop, window::WindowId};

type AcquireImageResult = (u32, bool, SwapchainAcquireFuture);

pub struct Terra {
    _instance: Arc<Instance>,
    gpu_resources: Rc<RefCell<GpuResources>>,
    _graphics_resources: Rc<RefCell<GraphicsResources>>,
    graphics_context: Rc<RefCell<GraphicsContext>>,
    sprite_program: SpriteRenderProgram,
}

impl Terra {
    pub fn init(events: &EventLoop<()>) -> Terra {
        let instance = util::create_instance(&util::create_library());
        let surface = util::create_surface(&instance, events);
        let gpu_resources = Rc::new(RefCell::new(GpuResources::init(&instance, &surface)));
        let graphics_resources = Rc::new(RefCell::new(GraphicsResources::new(&gpu_resources)));
        let graphics_context = Rc::new(RefCell::new(GraphicsContext::new(&graphics_resources)));
        let sprite_program =
            SpriteRenderProgram::new(&graphics_context, &graphics_resources, &gpu_resources);

        Terra {
            _instance: instance,
            gpu_resources,
            _graphics_resources: graphics_resources,
            graphics_context,
            sprite_program,
        }
    }
}

impl Terra {
    pub fn render(&mut self) {
        if let Some((image, mut suboptimal, acquire_future)) = self.acquire_swapchain_image() {
            let resources = self.gpu_resources.borrow();
            let framebuffer = &resources.frame_buffers()[image as usize];

            if let Some(command_buffer) = self.sprite_program.draw(&framebuffer) {
                let future = acquire_future
                    .then_execute(resources.queue().clone(), command_buffer)
                    .expect("Failed to execute command buffer")
                    .then_swapchain_present(
                        resources.queue().clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            resources.swapchain().clone(),
                            image,
                        ),
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
                let present_info = SwapchainPresentInfo::swapchain_image_index(
                    resources.swapchain().clone(),
                    image,
                );
                let _ = acquire_future
                    .then_swapchain_present(resources.queue().clone(), present_info)
                    .then_signal_fence_and_flush()
                    .unwrap()
                    .wait(None);
            }

            if !suboptimal {
                return;
            }
        }

        self.gpu_resources.borrow_mut().recreate_swapchain();
    }

    pub fn window_id(&self) -> WindowId {
        util::get_surface_window(&self.gpu_resources.borrow().surface()).id()
    }

    pub fn graphics_context(&self) -> &Rc<RefCell<GraphicsContext>> {
        &self.graphics_context
    }

    // Returns None if the swapchain needs to be recreated
    fn acquire_swapchain_image(&self) -> Option<AcquireImageResult> {
        let resources = self.gpu_resources.borrow();
        let swapchain = resources.swapchain();

        match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => Some(r),
            Err(AcquireError::OutOfDate) => None,
            Err(e) => panic!("Failed to acquire next swapchain image: {e}"),
        }
    }

    pub fn recreate_swapchain(&mut self) {
        self.gpu_resources.borrow_mut().recreate_swapchain()
    }
}
