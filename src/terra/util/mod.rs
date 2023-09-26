use std::sync::Arc;
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::Vertex as BaseVertex,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    shader::{EntryPoint, ShaderModule},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    Version, VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use super::vertex::Vertex;

pub fn create_library() -> Arc<VulkanLibrary> {
    VulkanLibrary::new().expect("Failed to create library.")
}

pub fn create_instance(library: &Arc<VulkanLibrary>) -> Arc<Instance> {
    let app_info = InstanceCreateInfo {
        application_name: Some("Hello Triangle".into()),
        application_version: Version {
            major: 1,
            minor: 0,
            patch: 0,
        },
        engine_name: Some("No Engine".into()),
        engine_version: Version {
            major: 1,
            minor: 0,
            patch: 0,
        },
        enabled_extensions: required_extensions(library, true),
        ..Default::default()
    };

    Instance::new(library.clone(), app_info).expect("failed to create Vulkan instance")
}

pub fn create_surface(instance: &Arc<Instance>, events: &EventLoop<()>) -> Arc<Surface> {
    WindowBuilder::new()
        .build_vk_surface(events, instance.clone())
        .expect("Failed to create Surface")
}

pub fn create_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
) -> (Arc<Device>, Arc<Queue>) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = get_phsyical_device(instance, surface);
    let create_info = DeviceCreateInfo {
        enabled_extensions: device_extensions,
        queue_create_infos: vec![QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        }],
        ..Default::default()
    };

    let (device, mut queues) =
        Device::new(physical_device, create_info).expect("Failed to create logical device.");

    let queue = queues.next().expect("Failed to obtain queue.");

    (device, queue)
}

pub fn create_command_pool(device: &Arc<Device>) -> Arc<StandardCommandBufferAllocator> {
    Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ))
}

pub fn create_memory_pool(device: &Arc<Device>) -> StandardMemoryAllocator {
    StandardMemoryAllocator::new_default(device.clone())
}

pub fn create_descriptor_set_pool(device: &Arc<Device>) -> Arc<StandardDescriptorSetAllocator> {
    Arc::new(StandardDescriptorSetAllocator::new(device.clone()))
}

pub fn create_swap_chain(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .expect("Failed to get surface capabilities.");

    let image_formats = device
        .physical_device()
        .surface_formats(surface, Default::default())
        .expect("Failed to get surface formats.");

    let window = get_surface_window(surface);

    let create_info = SwapchainCreateInfo {
        min_image_count: surface_capabilities.min_image_count,
        image_format: Some(image_formats[0].0),
        image_extent: window.inner_size().into(),
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        composite_alpha: surface_capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .expect("Failed to get compsite alpha"),
        ..Default::default()
    };

    Swapchain::new(device.clone(), surface.clone(), create_info)
        .expect("Failed to create swapchain.")
}

pub fn create_render_pass(device: &Arc<Device>, format: Format) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: format,
                samples: 1,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .expect("Failed to create render pass.")
}

pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    vertex_shader: &Arc<ShaderModule>,
    fragment_shader: &Arc<ShaderModule>,
) -> Arc<GraphicsPipeline> {
    let vs = get_shader_entry_point(vertex_shader);
    let fs = get_shader_entry_point(fragment_shader);
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

pub fn create_frame_buffers(
    render_targets: &Vec<Arc<SwapchainImage>>,
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    render_targets
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())
                .expect("Failed to create Image View for render target");
            let create_info = FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            };

            Framebuffer::new(render_pass.clone(), create_info)
                .expect("Failed to create Frame buffer")
        })
        .collect::<Vec<_>>()
}

pub fn get_surface_window(surface: &Arc<Surface>) -> &Window {
    surface.object().unwrap().downcast_ref::<Window>().unwrap()
}

pub fn get_surface_dimensions(surface: &Arc<Surface>) -> [u32; 2] {
    get_surface_window(surface).inner_size().into()
}

pub fn create_viewport(dimensions: [u32; 2]) -> Viewport {
    let x = dimensions[0] as f32;
    let y = dimensions[1] as f32;
    Viewport {
        origin: [0.0, 0.0],
        dimensions: [x, y],
        depth_range: 0.0..1.0,
    }
}

fn get_phsyical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
) -> (Arc<PhysicalDevice>, u32) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let devices = instance
        .enumerate_physical_devices()
        .expect("Failed to get physical devices.");

    devices
        .filter_map(|device| is_device_supported(&device, &device_extensions, surface))
        .min_by_key(|(device, _)| match device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("No suitable physical device found")
}

fn is_device_supported(
    device: &Arc<PhysicalDevice>,
    extensions: &DeviceExtensions,
    surface: &Arc<Surface>,
) -> Option<(Arc<PhysicalDevice>, u32)> {
    if !device.supported_extensions().contains(extensions) {
        return None;
    } else {
        if let Some(index) = device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(index, prop)| {
                prop.queue_flags.intersects(QueueFlags::GRAPHICS)
                    && device.surface_support(index as u32, surface).is_ok()
            })
        {
            return Some((device.clone(), index as u32));
        }
    }

    None
}

fn required_extensions(
    library: &Arc<VulkanLibrary>,
    enable_validation_layers: bool,
) -> InstanceExtensions {
    let mut extensions = vulkano_win::required_extensions(library);
    if enable_validation_layers {
        extensions.ext_debug_report = true;
    }

    extensions
}

fn get_shader_entry_point(shader: &Arc<ShaderModule>) -> EntryPoint<'_> {
    shader
        .entry_point("main")
        .expect("Failed to get vertex shader entry point.")
}