use std::{cell::RefCell, rc::Rc, sync::Arc};

use egui_winit_vulkano::{
    egui::{self, ScrollArea, TextureId},
    Gui, GuiConfig,
};
use vulkano::{
    device::Queue,
    format::Format,
    image::{view::ImageView, ImageUsage, StorageImage, SwapchainImage},
    memory::allocator::StandardMemoryAllocator,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    sampler::{Filter, SamplerAddressMode, SamplerCreateInfo},
    swapchain::{self, AcquireError, SwapchainAcquireFuture, SwapchainPresentInfo},
    sync::{FlushError, GpuFuture},
};

use winit::{event::WindowEvent, event_loop::EventLoop};

use crate::terra::{
    context::GraphicsContext,
    programs::sprite::SpriteRenderProgram,
    resources::{gpu::GpuResources, graphics::GraphicsResources},
    util,
};

type AcquireImageResult = (u32, bool, SwapchainAcquireFuture);

pub struct Editor {
    gui: Gui,
    resources: Rc<RefCell<GpuResources>>,
    swapchain_images: Vec<Arc<ImageView<SwapchainImage>>>,
    scene_image: Arc<ImageView<StorageImage>>,
    scene_frame_buffer: Arc<Framebuffer>,
    scene_image_dimensions: [u32; 2],
    scene_image_index: TextureId,
    sprite_render_program: SpriteRenderProgram,
    ctx: Rc<RefCell<GraphicsContext>>,
}

impl Editor {
    pub fn init(events: &EventLoop<()>) -> Editor {
        let vk_instance = util::create_instance(&util::create_library());
        let surface = util::create_surface(&vk_instance, events);

        let resources = GpuResources::init(&vk_instance, &surface);
        let swapchain_images = util::create_swapchain_image_views(resources.render_targets());
        let mut gui = Gui::new(
            events,
            surface.clone(),
            resources.queue().clone(),
            resources.swapchain().image_format(),
            GuiConfig {
                allow_srgb_render_target: true,
                ..Default::default()
            },
        );

        let scene_image_dimensions: [u32; 2] = [256; 2];
        let (scene_image, scene_frame_buffer) = create_scene_image(
            resources.memory_alloc(),
            resources.queue(),
            resources.render_pass(),
            scene_image_dimensions,
        );

        let scene_image_index =
            gui.register_user_image_view(scene_image.clone(), default_sampler());

        let resources = Rc::new(RefCell::new(resources));

        let graphics_resources = Rc::new(RefCell::new(GraphicsResources::new(&resources)));
        let graphics_ctx = Rc::new(RefCell::new(GraphicsContext::new(&graphics_resources)));
        let sprite_render_program =
            SpriteRenderProgram::new(&graphics_ctx, &graphics_resources, &resources);

        Editor {
            gui,
            resources,
            swapchain_images,
            scene_image,
            scene_frame_buffer,
            scene_image_dimensions,
            sprite_render_program,
            ctx: graphics_ctx,
            scene_image_index,
        }
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        util::get_surface_window(self.resources.borrow().surface()).id()
    }

    pub fn request_redraw(&self) {
        util::get_surface_window(self.resources.borrow().surface()).request_redraw();
    }

    pub fn update(&mut self, event: &WindowEvent) {
        self.gui.update(event);
    }

    pub fn ctx(&self) -> &Rc<RefCell<GraphicsContext>> {
        &self.ctx
    }

    fn render(&mut self) {
        let resources = self.resources.clone();
        let resources = resources.borrow();

        if let Some((index, mut suboptimal, acquire_future)) = self.acquire_swapchain_image() {
            let image = &self.swapchain_images[index as usize];
            let framebuffer = self.scene_frame_buffer.clone();

            if let Some(command_buffer) = self.sprite_render_program.draw(&framebuffer) {
                let draw_future = acquire_future
                    .then_execute(resources.queue().clone(), command_buffer)
                    .unwrap();

                let gui_future = self.gui.draw_on_image(draw_future, image.clone());

                let future = gui_future
                    .then_swapchain_present(
                        resources.queue().clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            resources.swapchain().clone(),
                            index,
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
            }

            if suboptimal {
                self.recreate_swapchain()
            }
        }
    }

    fn acquire_swapchain_image(&self) -> Option<AcquireImageResult> {
        let resources = self.resources.borrow();
        let swapchain = resources.swapchain();

        match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => Some(r),
            Err(AcquireError::OutOfDate) => None,
            Err(e) => panic!("Failed to acquire next swapchain image: {e}"),
        }
    }

    pub fn run(&mut self) {
        let resources = self.resources.clone();
        let resources = resources.borrow();

        let graphics_ctx = self.ctx.clone();
        let graphics_ctx = graphics_ctx.borrow();

        self.gui.immediate_ui(|gui| {
            let ctx = gui.context();

            ctx.set_visuals(egui::Visuals::dark());

            egui::SidePanel::right("Side Panel Right")
                .min_width(200.0)
                .show(&ctx, |ui| {
                    ui.heading("Inspector");
                    ui.separator();
                });

            egui::TopBottomPanel::bottom("Bottom Panel")
                .min_height(200.0)
                .show(&ctx, |ui| {
                    ui.columns(2, |columns| {
                        ScrollArea::vertical()
                            .id_source("asset-folders")
                            .show(&mut columns[0], |ui| {});

                        ScrollArea::vertical()
                            .id_source("asset-folders")
                            .show(&mut columns[1], |ui| {});
                    })
                });

            egui::SidePanel::left("Side Panel")
                .min_width(200.0)
                .show(&ctx, |ui| {
                    ui.heading("Hierarchy");
                    ui.separator();

                    for (_, renderers) in graphics_ctx.sprite_renderers() {
                        for renderer in renderers.iter() {
                            ui.label(renderer.borrow().name().as_ref());
                        }
                    }
                });

            egui::CentralPanel::default().show(&ctx, |ui| {
                let width = ui.available_width().ceil() as u32;
                let height = ui.available_height().ceil() as u32;

                if width != self.scene_image_dimensions[0]
                    || height != self.scene_image_dimensions[1]
                {
                    let (scene_image, scene_frame_buffer) = create_scene_image(
                        resources.memory_alloc(),
                        resources.queue(),
                        resources.render_pass(),
                        [width, height],
                    );

                    self.scene_image = scene_image.clone();
                    self.scene_frame_buffer = scene_frame_buffer;
                    self.scene_image_dimensions = [width, height];
                    gui.unregister_user_image(self.scene_image_index);
                    self.scene_image_index =
                        gui.register_user_image_view(scene_image.clone(), default_sampler());
                }

                ui.image(self.scene_image_index, [width as f32, height as f32]);
            });
        });

        self.render();
    }

    pub fn recreate_swapchain(&mut self) {
        let mut resources = self.resources.borrow_mut();

        resources.recreate_swapchain();
        self.swapchain_images = util::create_swapchain_image_views(resources.render_targets());
    }
}

fn create_scene_image(
    memory_allocator: &StandardMemoryAllocator,
    queue: &Arc<Queue>,
    render_pass: &Arc<RenderPass>,
    dimensions: [u32; 2],
) -> (Arc<ImageView<StorageImage>>, Arc<Framebuffer>) {
    let scene_image = StorageImage::general_purpose_image_view(
        memory_allocator,
        queue.clone(),
        dimensions,
        Format::B8G8R8A8_SRGB,
        ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT,
    )
    .unwrap();

    let frame_buffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![scene_image.clone()],
            ..Default::default()
        },
    )
    .expect("Failed to create frame buffer for scene image");

    (scene_image, frame_buffer)
}

fn default_sampler() -> SamplerCreateInfo {
    SamplerCreateInfo {
        mag_filter: Filter::Linear,
        min_filter: Filter::Linear,
        address_mode: [SamplerAddressMode::ClampToBorder; 3],
        ..Default::default()
    }
}
