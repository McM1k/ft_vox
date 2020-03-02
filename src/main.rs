use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::swapchain::{FullscreenExclusive, ColorSpace};
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

use winit::window::{WindowBuilder, Window};
use winit::event_loop::EventLoop;
use winit::event::{Event, WindowEvent};

use std::sync::Arc;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;


fn main() {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).unwrap()
    };

    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    let mut events_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
    let window = surface.window();

    let queue_family = physical.queue_families().find(|&q| {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).unwrap();

    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none()};
    let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext, [(queue_family, 0.5)].iter().cloned()).unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let initial_dimensions = {
            let dimensions= window.inner_size();
            let scale_factor = window.scale_factor();
            [(dimensions.width as f64 * scale_factor) as u32, (dimensions.height as f64 * scale_factor) as u32]
        };

        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
            initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha,
            PresentMode::Fifo, FullscreenExclusive::Default, false, ColorSpace::SrgbNonLinear).unwrap()
    };

    let vertex_buffer = {
        #[derive(Default, Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, [
            Vertex { position: [-0.5, -0.25] },
            Vertex { position: [0.0, 0.5] },
            Vertex { position: [0.25, -0.1] }
        ].iter().cloned()).unwrap()
    };

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(
        device.clone(),
        Attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {},
        }
    ).unwrap());

    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer()
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
        compare_mask: None,
        write_mask: None,
        reference: None
    };
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    
    
}


