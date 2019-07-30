use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::sync::GpuFuture;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::format::Format;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::format::ClearValue;
use vulkano::framebuffer::Framebuffer;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::framebuffer::Subpass;
use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

fn copy_buffer() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
                             .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }
    let queue_family = physical.queue_families().find(|&q| q.supports_graphics())
                       .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let source_content = 0 .. 64;
    let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                            source_content).expect("failed to create buffer");
    let dest_content = (0 .. 64).map(|_| 0);
    let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                          dest_content).expect("failed to create buffer");
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
                         .copy_buffer(source.clone(), dest.clone()).unwrap()
                         .build().unwrap();
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    let src_content = source.read().unwrap();
    let dest_content = dest.read().unwrap();
    println!("Source content starts...");
    for i in 0..src_content.len() {
        print!("{} ", src_content[i]);
    }
    println!();
    println!("Destination content starts...");
    for i in 0..dest_content.len() {
        print!("{} ", dest_content[i]);
    }
    println!();
}

fn hello_shader() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
                             .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }
    let queue_family = physical.queue_families().find(|&q| q.supports_graphics())
                       .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let data_iter = 0 .. 65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                                 data_iter).expect("failed to create buffer");

    mod cs {
        vulkano_shaders::shader!{
            ty: "compute",
            src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
        }
    }

    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
    let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
                                        .expect("failed to create compute pipeline"));
    let set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
                       .add_buffer(data_buffer.clone()).unwrap().build().unwrap());
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
                         .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
                         .build().unwrap();
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    let content = data_buffer.read().unwrap();
    println!("Start printing the data buffer (65536 elements)...");
    for i in 0..content.len() {
        print!("{} ", content[i]);
    }
    println!();
}

fn hello_image() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
                             .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }
    let queue_family = physical.queue_families().find(|&q| q.supports_graphics())
                       .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 },
                                  Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0 .. 1024 * 1024 * 4).map(|_| 0u8))
                                             .expect("failed to create buffer");
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
            .clear_color_image(image.clone(), ClearValue::Float([1.0, 1.0, 0.0, 1.0])).unwrap()
            .copy_image_to_buffer(image.clone(), buf.clone()).unwrap()
            .build().unwrap();
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

fn hello_graphics() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
                             .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }
    let queue_family = physical.queue_families().find(|&q| q.supports_graphics())
                       .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };
    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                                       vec![vertex1, vertex2, vertex3].into_iter()).unwrap();

    mod vs {
        vulkano_shaders::shader!{
            ty: "vertex",
            src: "
#version 450
layout(location = 0) in vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
        }
    }

    mod fs {
        vulkano_shaders::shader!{
            ty: "fragment",
            src: "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
        }
    }

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 },
                                  Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
    let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
                                             (0 .. 1024 * 1024 * 4).map(|_| 0u8))
                                             .expect("failed to create buffer");
    let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
                               .add(image.clone()).unwrap()
                               .build().unwrap());
    let pipeline = Arc::new(GraphicsPipeline::start()
        // Defines what kind of vertex input is expected.
        .vertex_input_single_buffer::<Vertex>()
        // The vertex shader.
        .vertex_shader(vs.main_entry_point(), ())
        // Defines the viewport (explanations below).
        .viewports_dynamic_scissors_irrelevant(1)
        // The fragment shader.
        .fragment_shader(fs.main_entry_point(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(device.clone())
        .unwrap());
    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [1024.0, 1024.0],
            depth_range: 0.0 .. 1.0,
        }]),
        .. DynamicState::none()
    };

    let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
        .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
        .unwrap()

        .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
        .unwrap()

        .end_render_pass()
        .unwrap()

        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap()

        .build()
        .unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("triangle.png").unwrap();
}

fn main() {
    // copy_buffer();
    // hello_shader();
    // hello_image();
    hello_graphics();
}
