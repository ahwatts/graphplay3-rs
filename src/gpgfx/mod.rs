use gfx::{Resources, Slice, IntoIndexBuffer};
use gfx::handle::Buffer;
use gfx::traits::{FactoryExt, Pod};
use gfx::pso::buffer::Structure;
use gfx::format::Format;

pub struct Geometry<V: Pod + Structure<Format>, R: Resources> {
    vertices: Buffer<R, V>,
    indices: Slice<R>,
}

impl<V: Pod + Structure<Format>, R: Resources> Geometry<V, R> {
    pub fn new<F, I>(factory: &mut F, vertices: &[V], elems: I) -> Geometry<V, R>
        where F: FactoryExt<R>, I: IntoIndexBuffer<R>
    {
        let (vbuf, elems) = factory.create_vertex_buffer_with_slice(vertices, elems);
        Geometry {
            vertices: vbuf,
            indices: elems,
        }
    }
}
