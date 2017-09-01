extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra;

#[macro_use]
extern crate gfx;

use gfx::{Device, Encoder};
use gfx::traits::FactoryExt;
use glutin::{ContextBuilder, EventsLoop, GlContext, VirtualKeyCode, WindowBuilder};
use nalgebra::*;
use std::thread;
use std::time::{Duration, Instant};

type ColorType = gfx::format::Rgba8;
type DepthType = gfx::format::Depth;

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
        out_color: gfx::RenderTarget<ColorType> = "FragColor",
        out_depth: gfx::DepthTarget<DepthType> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

const OCTOHEDRON_VERTICES: &[Vertex] = &[
    Vertex { position: [  1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ] },
    Vertex { position: [ -1.0,  0.0,  0.0, ], color: [ 1.0, 0.0, 0.0, 1.0 ] },
    Vertex { position: [  0.0,  0.0,  1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ] },
    Vertex { position: [  0.0,  0.0, -1.0, ], color: [ 0.0, 0.0, 1.0, 1.0 ] },
    Vertex { position: [  0.0, -1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ] },
    Vertex { position: [  0.0,  1.0,  0.0, ], color: [ 0.0, 1.0, 0.0, 1.0 ] },
];

const OCTOHEDRON_ELEMENTS: &[u32] = &[
    4, 0, 2, 4, 3, 0, 4, 1, 3, 4, 2, 1,
    5, 2, 0, 5, 0, 3, 5, 3, 1, 5, 1, 2,
];

const FRAME_PERIOD: f32 = 1.0 / 60.0;
// const TIME_STEP: f32 = 1.0 / 300.0;

fn main() {
    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("graphplay3")
        .with_dimensions(800, 600);
    let context_builder = ContextBuilder::new();

    let (
        window,
        mut device,
        mut factory,
        render_target,
        depth_target
    ) = gfx_window_glutin::init::<ColorType, DepthType>(window_builder, context_builder, &events_loop);

    let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_vertex.glsl")),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/unlit_fragment.glsl")),
        unlit_pipe::new(),
    ).unwrap();
    let (octo_vbuf, octo_elems) = factory.create_vertex_buffer_with_slice(OCTOHEDRON_VERTICES, OCTOHEDRON_ELEMENTS);
    let constant_buffer = factory.create_constant_buffer::<ViewAndProjection>(1);

    let mut data = unlit_pipe::Data {
        vertices: octo_vbuf,
        model: Matrix4::identity().into(),
        view_and_projection: constant_buffer,
        out_color: render_target,
        out_depth: depth_target,
    };

    let mut angle = 0.0;

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

    // Misc. loop variables.
    let mut prev_time = Instant::now();
    let pi = std::f32::consts::PI;
    let frame_period = Duration::new(0, (FRAME_PERIOD * 1.0e9) as u32);

    let mut frame_count = 0;
    let mut avg_update_secs = 0.0;
    let mut avg_sleep_secs = 0.0;
    let mut avg_real_sleep_secs = 0.0;

    let mut running = true;
    while running {
        // Get the elapsed time.
        let time = Instant::now();
        let elapsed = time.duration_since(prev_time);
        prev_time = time;
        let secs = elapsed.as_secs() as f32;
        let subsecs = elapsed.subsec_nanos() as f32 / 1.0e9;
        let ftime = secs + subsecs;

        events_loop.poll_events(|event| {
            use glutin::Event::*;
            match event {
                WindowEvent { event: window_event, .. } => {
                    use glutin::WindowEvent::*;
                    match window_event {
                        KeyboardInput { input: glutin::KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } => {
                            running = false;
                        },
                        Closed => {
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
                },
                _ => {},
            }
        });

        angle += ((2.0*pi) / (600.0*FRAME_PERIOD)) * ftime;
        if angle > 2.0*pi {
            angle -= 2.0*pi;
        }
        let model_matrix = Rotation3::from_euler_angles(angle, angle / 2.0, 0.0);

        encoder.clear(&mut data.out_color, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear_depth(&mut data.out_depth, 1.0);
        data.model = model_matrix.to_homogeneous().into();
        encoder.update_constant_buffer(&data.view_and_projection, &vp_elem);
        encoder.draw(&octo_elems, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        let update_time = Instant::now();
        let update_duration = update_time.duration_since(time);

        frame_count += 1;
        let update_secs = (update_duration.as_secs() as f32) +
            (update_duration.subsec_nanos() as f32 / 1.0e9);
        let frame_count_float: f32 = frame_count as f32;
        avg_update_secs = avg_update_secs + ((update_secs - avg_update_secs) / frame_count_float);

        if update_duration < frame_period {
            let sleep_duration = frame_period - update_duration;
            let sleep_secs = (sleep_duration.as_secs() as f32) +
                (sleep_duration.subsec_nanos() as f32 / 1.0e9);
            avg_sleep_secs = avg_sleep_secs + ((sleep_secs - avg_sleep_secs) / frame_count_float);

            thread::sleep(sleep_duration);

            let real_sleep_time = Instant::now();
            let real_sleep_duration = real_sleep_time - update_time;
            let real_sleep_secs = (real_sleep_duration.as_secs() as f32) +
                (real_sleep_duration.subsec_nanos() as f32 / 1.0e9);
            avg_real_sleep_secs = avg_real_sleep_secs + ((real_sleep_secs - avg_real_sleep_secs) / frame_count_float);
        }
    }
}
