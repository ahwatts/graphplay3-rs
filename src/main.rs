extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra;

#[macro_use]
extern crate gfx;

use gfx::{Device, Encoder};
use gfx::format::{Srgba8, Depth};
use gfx::traits::FactoryExt;
use glutin::{Event, EventsLoop, VirtualKeyCode, WindowBuilder};
use nalgebra::*;

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
        out_color: gfx::RenderTarget<Srgba8> = "FragColor",
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
        render_target,
        depth_target
    ) = gfx_window_glutin::init::<Srgba8, Depth>(builder, &events_loop);

    let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();

    let octohedron_vertices: &[Vertex] = &[
        Vertex { position: [  1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ], },
        Vertex { position: [ -1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ], },
        Vertex { position: [  0.0,  0.0,  1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ], },
        Vertex { position: [  0.0,  0.0, -1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ], },
        Vertex { position: [  0.0, -1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ], },
        Vertex { position: [  0.0,  1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ], },
    ];

    let octohedron_elements: &[u32] = &[
        4, 0, 2, 4, 3, 0, 4, 1, 3, 4, 2, 1,
        5, 2, 0, 5, 0, 3, 5, 3, 1, 5, 1, 2,
    ];

    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_vertex.glsl")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_fragment.glsl")),
        unlit_pipe::new(),
    ).unwrap();
    let (octo_vbuf, octo_elems) = factory.create_vertex_buffer_with_slice(octohedron_vertices, octohedron_elements);
    let constant_buffer = factory.create_constant_buffer::<ViewAndProjection>(1);

    let mut data = unlit_pipe::Data {
        vertices: octo_vbuf,
        model: Matrix4::identity().into(),
        view_and_projection: constant_buffer,
        out_color: render_target,
        out_depth: depth_target,
    };

    let view_matrix = Isometry3::look_at_rh(
        &Point3::new(0.0, 0.0, 10.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0),
    );

    let (width, height, _, _) = data.out_color.get_dimensions();
    let mut proj_matrix = Perspective3::new(
        width as f32 / height as f32,
        3.14 / 6.0,
        0.01,
        100.0,
    );

    let mut vp_elem = ViewAndProjection {
        view: view_matrix.to_homogeneous().into(),
        view_inv: view_matrix.inverse().to_homogeneous().into(),
        projection: proj_matrix.to_homogeneous().into(),
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
                    let (width, height, _, _) = data.out_color.get_dimensions();
                    proj_matrix = Perspective3::new(
                        width as f32 / height as f32,
                        3.14 / 6.0,
                        0.01,
                        100.0,
                    );
                    vp_elem.projection = proj_matrix.to_homogeneous().into();
                },
                _ => {},
            }
        });

        encoder.clear(&mut data.out_color, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear_depth(&mut data.out_depth, 1.0);
        encoder.update_constant_buffer(&data.view_and_projection, &vp_elem);
        encoder.draw(&octo_elems, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
