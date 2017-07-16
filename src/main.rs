extern crate gfx_window_glutin;
extern crate glutin;

#[macro_use]
extern crate gfx;

use gfx::{Device, Encoder, Global};
use gfx::format::{Srgba8, Depth};
use gfx::pso::DataLink;
use gfx::traits::FactoryExt;
use glutin::{Event, EventsLoop, VirtualKeyCode, WindowBuilder};

gfx_defines! {
    vertex Vertex {
        position: [f32; 3] = "position",
        color: [f32; 4] = "color",
    }

    constant ViewAndProjection {
        view: [[f32; 4]; 4] = "view",
        view_inv: [[f32; 4]; 4] = "view_inv",
        projection: [[f32; 4]; 4] = "projection",
    }

    pipeline unlit_pipe {
        vertices: gfx::VertexBuffer<Vertex> = (),
        model: gfx::Global<[[f32; 4]; 4]> = "model",
        view_and_projection: gfx::ConstantBuffer<ViewAndProjection> = "view_and_projection",
        out_color: gfx::RenderTarget<Srgba8> = "Target0",
        out_depth: gfx::DepthTarget<Depth> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

fn main() {
    let events_loop = EventsLoop::new();
    let builder = WindowBuilder::new()
        .with_title("graphplay3")
        .with_dimensions(800, 600);

    let (
        window,
        mut device,
        mut factory,
        mut render_target,
        mut depth_target
    ) = gfx_window_glutin::init::<Srgba8, Depth>(builder, &events_loop);

    let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();

    let octohedron = [
        Vertex { position: [  1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ], },
        Vertex { position: [ -1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ], },
        Vertex { position: [  0.0,  0.0,  1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ], },
        Vertex { position: [  0.0,  0.0, -1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ], },
        Vertex { position: [  0.0, -1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ], },
        Vertex { position: [  0.0,  1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ], },
    ];

    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_vertex.glsl")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_fragment.glsl")),
        unlit_pipe::new(),
    );
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&octohedron, ());
    let constant_buffer = factory.create_constant_buffer::<ViewAndProjection>(1);

    let mut data = unlit_pipe::Data {
        vertices: vertex_buffer,
        model: Default::default(),
        view_and_projection: constant_buffer,
        out_color: render_target,
        out_depth: depth_target,
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|Event::WindowEvent { window_id: _, event }| {
            use glutin::WindowEvent::*;
            match event {
                KeyboardInput(_, _, Some(VirtualKeyCode::Escape), _) | Closed => {
                    running = false;
                },
                Resized(_, _) => {
                    gfx_window_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
                },
                _ => {},
            }
        });

        encoder.clear(&mut data.out_color, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear_depth(&mut data.out_depth, 0.0);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
