use gfx::{VertexBuffer, Resources, Slice, IntoIndexBuffer};
use gfx::traits::{FactoryExt, Pod};
use gfx::pso::buffer::Structure;
use gfx::format::Format;

pub struct Geometry<V: Pod + Structure<Format>, R: Resources> {
    vertices: VertexBuffer<V>,
    indices: Slice<R>,
}

impl<V: Pod + Structure<Format>, R: Resources> Geometry<V, R> {
    pub fn new<F: FactoryExt<R>, I: IntoIndexBuffer<R>>(factory: &mut F, vertices: &[V], elems: I) {
        let (vbuf, elems) = factory.create_vertex_buffer_with_slice(vertices, elems);
    }
}
