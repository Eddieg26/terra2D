pub mod camera;
pub mod sprite;
pub mod terra;
pub mod transform;

use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

use camera::Camera;
use sprite::SpriteRenderer;
use terra::Terra;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

fn main() {
    let events = EventLoop::new();

    let mut terra = Terra::init(&events);
    let id = terra.window_id();

    let camera = Rc::new(RefCell::new(Camera::new()));

    if let Some(sprite) = terra
        .sprites()
        .get(&get_id("./src/assets/sprites/094 - Copy.png"))
    {
        let gengar = Rc::new(RefCell::new(SpriteRenderer::new(sprite)));
        gengar.borrow_mut().color_mut().set(1.0, 1.0, 1.0);
        terra.add_sprite_renderer(&gengar);
        terra.set_camera(&camera);
    }

    events.run(move |event, _, flow| {
        flow.set_wait();

        match event {
            Event::WindowEvent { event, window_id } if window_id == id => match event {
                WindowEvent::CloseRequested => flow.set_exit(),
                WindowEvent::Resized(_) => {
                    terra.recreate_swapchain();
                }
                _ => (),
            },

            Event::RedrawEventsCleared => terra.render(),

            _ => (),
        }
    })
}

fn get_id(path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);

    hasher.finish()
}

// Copyright (c) 2022 taidaesal
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>
//
// This file contains code copied and/or adapted
// from code provided by the Vulkano project under
// the MIT license

// use nalgebra_glm::{self as glm, Mat4};
// use std::sync::Arc;
// use terra::renderer::GlobalData;
// use terra::shader::loader::ShaderLoader;
// use vulkano::buffer::{BufferContents, Subbuffer};
// use vulkano::command_buffer::PrimaryCommandBufferAbstract;
// use vulkano::descriptor_set::layout::DescriptorSetLayout;
// use vulkano::device::physical::PhysicalDeviceType;
// use vulkano::device::{DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
// use vulkano::image::{AttachmentImage, MipmapsCount, SwapchainImage};
// use vulkano::instance::{Instance, InstanceCreateInfo};
// use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
// use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
// use vulkano::pipeline::Pipeline;
// use vulkano::render_pass::{FramebufferCreateInfo, Subpass};
// use vulkano::sampler::SamplerMipmapMode;
// use vulkano::swapchain::{
//     self, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
//     SwapchainPresentInfo,
// };
// use vulkano::sync::{self, FlushError, GpuFuture};
// use vulkano::{
//     buffer::{Buffer, BufferCreateInfo, BufferUsage},
//     command_buffer::{
//         allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
//         RenderPassBeginInfo, SubpassContents,
//     },
//     descriptor_set::{
//         allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
//     },
//     device::Device,
//     format::Format,
//     image::{view::ImageView, ImageDimensions, ImmutableImage},
//     memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
//     pipeline::{
//         graphics::{input_assembly::InputAssemblyState, viewport::Viewport},
//         graphics::{vertex_input::Vertex as BaseVertex, viewport::ViewportState},
//         GraphicsPipeline, PipelineBindPoint,
//     },
//     render_pass::{Framebuffer, RenderPass},
//     sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
// };
// use vulkano::{Version, VulkanLibrary};

// use vulkano_win::VkSurfaceBuild;

// use winit::event::{Event, WindowEvent};
// use winit::event_loop::{ControlFlow, EventLoop};
// use winit::window::{Window, WindowBuilder};

// use png;

// use std::io::Cursor;

// use crate::terra::util;
// use crate::terra::vertex::Vertex;

// #[derive(Debug, Clone, BufferContents)]
// #[repr(C)]
// struct MVP {
//     model: [f32; 16],
//     view: [f32; 16],
//     projection: [f32; 16],
// }

// impl MVP {
//     fn new() -> MVP {
//         let identity = mat4_to_array(Mat4::identity());
//         MVP {
//             model: identity,
//             view: identity,
//             projection: identity,
//         }
//     }
// }

// fn mat4_to_array(matrix: Mat4) -> [f32; 16] {
//     let mut mat = [0.0; 16];

//     let matrix = matrix.data.as_slice();
//     for index in 0..16 {
//         mat[index] = matrix[index];
//     }

//     mat
// }

// mod vs {
//     vulkano_shaders::shader! {
//         ty: "vertex",
//         src: "
//             #version 450
//             layout(location = 0) in vec4 vertex;

//             layout(location = 0) out vec2 tex_coords;

//             layout(set = 0, binding = 0) uniform PerCamera {
//                 mat4 projection;
//                 mat4 view;
//             }camera;

//             void main() {
//                 vec4 pos = camera.view * vec4(1.0);
//                 gl_Position = vec4(vertex.xy, 0.0, 1.0);
//                 tex_coords = vertex.zw;
//             }
//         ",
//     }
// }

// mod fs {
//     vulkano_shaders::shader! {
//         ty: "fragment",
//         src: "
//             #version 450
//             layout(location = 0) in vec2 tex_coords;

//             layout(location = 0) out vec4 f_color;

//             layout(set = 1, binding = 0) uniform sampler2D tex;

//             void main() {
//                 f_color = texture(tex, tex_coords);
//             }
//         "
//     }
// }
// fn create_global_descriptor_set(
//     allocator: &StandardDescriptorSetAllocator,
//     layout: &Arc<DescriptorSetLayout>,
//     buffer: &Subbuffer<GlobalData>,
// ) -> Arc<PersistentDescriptorSet> {
//     PersistentDescriptorSet::new(
//         allocator,
//         layout.clone(),
//         [WriteDescriptorSet::buffer(0, buffer.clone())],
//     )
//     .expect("Failed to create global descriptor set")
// }

// fn create_global_buffer(allocator: &StandardMemoryAllocator) -> Subbuffer<GlobalData> {
//     Buffer::from_data(
//         allocator,
//         BufferCreateInfo {
//             usage: BufferUsage::UNIFORM_BUFFER,
//             ..Default::default()
//         },
//         AllocationCreateInfo {
//             usage: MemoryUsage::Upload,
//             ..Default::default()
//         },
//         GlobalData::new(Mat4::identity(), Mat4::identity()),
//     )
//     .expect("Failed to create Global Buffer")
// }

// fn main() {
//     let mut mvp = MVP::new();
//     let event_loop = EventLoop::new();
//     let instance = util::create_instance(&util::create_library());
//     let surface = util::create_surface(&instance, &event_loop);
//     let (device, queue) = util::create_device(&instance, &surface);
//     let shaders = ShaderLoader::load(&device);

//     let command_buffer_allocator =
//         StandardCommandBufferAllocator::new(device.clone(), Default::default());
//     let descriptor_set_allocator = util::create_descriptor_set_pool(&device);
//     let vs = shaders.vertex("sprite").unwrap();
//     let fs = shaders.fragment("sprite").unwrap();
//     let (mut swapchain, images) = util::create_swap_chain(&device, &surface);
//     let render_pass = util::create_render_pass(&device, swapchain.image_format());

//     let pipeline = GraphicsPipeline::start()
//         .vertex_input_state(Vertex::per_vertex())
//         .vertex_shader(vs.entry_point("main").unwrap(), ())
//         .input_assembly_state(InputAssemblyState::new())
//         .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
//         .fragment_shader(fs.entry_point("main").unwrap(), ())
//         .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
//         .build(device.clone())
//         .unwrap();

//     let memory_allocator = util::create_memory_pool(&device);
//     let mut framebuffers = util::create_frame_buffers(&images, &render_pass);

//     let global_buffer = create_global_buffer(&memory_allocator);
//     let _global_descriptor_set = create_global_descriptor_set(
//         &descriptor_set_allocator,
//         &pipeline.layout().set_layouts()[0],
//         &global_buffer,
//     );

//     let w_mult = 1.0;
//     let h_mult = 1.0;

//     let indices: [u32; 6] = [0, 1, 2, 1, 0, 3];
//     let vertices = [
//         Vertex {
//             vertex: [-0.5 * w_mult, 0.5 * h_mult, 0.0, 1.0],
//         },
//         Vertex {
//             vertex: [0.5 * w_mult, -0.5 * h_mult, 1.0, 0.0],
//         },
//         Vertex {
//             vertex: [-0.5 * w_mult, -0.5 * h_mult, 0.0, 0.0],
//         },
//         Vertex {
//             vertex: [0.5 * w_mult, 0.5 * h_mult, 1.0, 1.0],
//         },
//     ];

//     let vertex_buffer = Buffer::from_iter(
//         &memory_allocator,
//         BufferCreateInfo {
//             usage: BufferUsage::VERTEX_BUFFER,
//             ..Default::default()
//         },
//         AllocationCreateInfo {
//             usage: MemoryUsage::Upload,
//             ..Default::default()
//         },
//         vertices.into_iter(),
//     )
//     .expect("Failed to create destination buffer ");

//     let index_buffer = Buffer::from_iter(
//         &memory_allocator,
//         BufferCreateInfo {
//             usage: BufferUsage::INDEX_BUFFER,
//             ..Default::default()
//         },
//         AllocationCreateInfo {
//             usage: MemoryUsage::Upload,
//             ..Default::default()
//         },
//         indices.into_iter(),
//     )
//     .expect("Failed to create destination buffer ");

//     let mut viewport = Viewport {
//         origin: [0.0, 0.0],
//         dimensions: [0.0, 0.0],
//         depth_range: 0.0..1.0,
//     };

//     let sampler = Sampler::new(
//         device.clone(),
//         SamplerCreateInfo {
//             mag_filter: Filter::Linear,
//             min_filter: Filter::Linear,
//             mipmap_mode: SamplerMipmapMode::Nearest,
//             address_mode: [SamplerAddressMode::Repeat; 3],
//             mip_lod_bias: 0.0,
//             ..Default::default()
//         },
//     )
//     .unwrap();

//     // load the image data and dimensions before event loop
//     let (image_data, image_dimensions) = {
//         let png_bytes = include_bytes!("./assets/sprites/094 - Copy.png").to_vec();
//         let cursor = Cursor::new(png_bytes);
//         let decoder = png::Decoder::new(cursor);
//         let mut reader = decoder.read_info().unwrap();
//         let info = reader.info();
//         let image_dimensions = ImageDimensions::Dim2d {
//             width: info.width,
//             height: info.height,
//             array_layers: 1,
//         };
//         let mut image_data = Vec::new();
//         let depth: u32 = match info.bit_depth {
//             png::BitDepth::One => 1,
//             png::BitDepth::Two => 2,
//             png::BitDepth::Four => 4,
//             png::BitDepth::Eight => 8,
//             png::BitDepth::Sixteen => 16,
//         };
//         image_data.resize((info.width * info.height * depth) as usize, 0);
//         reader.next_frame(&mut image_data).unwrap();
//         (image_data, image_dimensions)
//     };

//     let texture = create_texture(
//         &command_buffer_allocator,
//         &memory_allocator,
//         &queue,
//         &image_data,
//         image_dimensions,
//     );
//     let texture_set = create_set_from_texture(
//         &descriptor_set_allocator,
//         &pipeline.layout().set_layouts()[1],
//         &texture,
//         &sampler,
//     );

//     let mut recreate_swapchain = false;

//     let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

//     event_loop.run(move |event, _, control_flow| match event {
//         Event::WindowEvent {
//             event: WindowEvent::CloseRequested,
//             ..
//         } => {
//             *control_flow = ControlFlow::Exit;
//         }
//         Event::WindowEvent {
//             event: WindowEvent::Resized(_),
//             ..
//         } => {
//             recreate_swapchain = true;
//         }
//         Event::RedrawEventsCleared => {
//             previous_frame_end
//                 .as_mut()
//                 .take()
//                 .unwrap()
//                 .cleanup_finished();

//             if recreate_swapchain {
//                 let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
//                 let image_extent: [u32; 2] = window.inner_size().into();

//                 mvp.projection = mat4_to_array(glm::ortho(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0));

//                 let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
//                     image_extent,
//                     ..swapchain.create_info()
//                 }) {
//                     Ok(r) => r,
//                     Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
//                     Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
//                 };

//                 swapchain = new_swapchain;
//                 framebuffers = util::create_frame_buffers(&new_images, &render_pass);
//                 viewport = util::create_viewport(util::get_surface_dimensions(&surface));
//                 recreate_swapchain = false;
//             }

//             let (image_index, suboptimal, acquire_future) =
//                 match swapchain::acquire_next_image(swapchain.clone(), None) {
//                     Ok(r) => r,
//                     Err(AcquireError::OutOfDate) => {
//                         recreate_swapchain = true;
//                         return;
//                     }
//                     Err(e) => panic!("Failed to acquire next image: {:?}", e),
//                 };

//             if suboptimal {
//                 recreate_swapchain = true;
//             }

//             let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
//                 &command_buffer_allocator,
//                 queue.queue_family_index(),
//                 CommandBufferUsage::OneTimeSubmit,
//             )
//             .unwrap();

//             let clear_values = vec![Some([0.0, 0.68, 1.0, 1.0].into()), Some(1.0.into())];

//             cmd_buffer_builder
//                 .begin_render_pass(
//                     RenderPassBeginInfo {
//                         clear_values,
//                         ..RenderPassBeginInfo::framebuffer(
//                             framebuffers[image_index as usize].clone(),
//                         )
//                     },
//                     SubpassContents::Inline,
//                 )
//                 .unwrap()
//                 .set_viewport(0, [viewport.clone()])
//                 .bind_pipeline_graphics(pipeline.clone())
//                 .bind_descriptor_sets(
//                     PipelineBindPoint::Graphics,
//                     pipeline.layout().clone(),
//                     0,
//                     texture_set.clone(),
//                 )
//                 .bind_vertex_buffers(0, vertex_buffer.clone())
//                 .bind_index_buffer(index_buffer.clone())
//                 .draw_indexed(6, 1, 0, 0, 0)
//                 .unwrap()
//                 .end_render_pass()
//                 .unwrap();

//             let command_buffer = cmd_buffer_builder.build().unwrap();

//             let future = previous_frame_end
//                 .take()
//                 .unwrap()
//                 .join(acquire_future)
//                 .then_execute(queue.clone(), command_buffer)
//                 .unwrap()
//                 .then_swapchain_present(
//                     queue.clone(),
//                     SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
//                 )
//                 .then_signal_fence_and_flush();

//             match future {
//                 Ok(future) => {
//                     previous_frame_end = Some(Box::new(future) as Box<_>);
//                 }
//                 Err(FlushError::OutOfDate) => {
//                     recreate_swapchain = true;
//                     previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
//                 }
//                 Err(e) => {
//                     println!("Failed to flush future: {:?}", e);
//                     previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
//                 }
//             }
//         }
//         _ => (),
//     });
// }

// fn create_texture(
//     command_buffer_allocator: &StandardCommandBufferAllocator,
//     allocator: &StandardMemoryAllocator,
//     queue: &Arc<Queue>,
//     image_data: &Vec<u8>,
//     dimensions: ImageDimensions,
// ) -> Arc<ImageView<ImmutableImage>> {
//     let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
//         command_buffer_allocator,
//         queue.queue_family_index(),
//         CommandBufferUsage::OneTimeSubmit,
//     )
//     .unwrap();

//     let image = ImmutableImage::from_iter(
//         allocator,
//         image_data.iter().cloned(),
//         dimensions,
//         MipmapsCount::One,
//         Format::R8G8B8A8_SRGB,
//         &mut cmd_buffer_builder,
//     )
//     .unwrap();

//     let image_view = ImageView::new_default(image).unwrap();

//     let command_buffer = cmd_buffer_builder.build();

//     let _future = command_buffer.unwrap().execute(queue.clone());

//     image_view
// }

// fn create_set_from_texture(
//     descriptor_set_alloc: &StandardDescriptorSetAllocator,
//     layout: &Arc<DescriptorSetLayout>,
//     image_view: &Arc<ImageView<ImmutableImage>>,
//     sampler: &Arc<Sampler>,
// ) -> Arc<PersistentDescriptorSet> {
//     PersistentDescriptorSet::new(
//         descriptor_set_alloc,
//         layout.clone(),
//         [WriteDescriptorSet::image_view_sampler(
//             0,
//             image_view.clone(),
//             sampler.clone(),
//         )],
//     )
//     .expect("Failed to create image layout")
// }
