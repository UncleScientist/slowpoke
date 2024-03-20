use lyon_tessellation::{
    geometry_builder::simple_builder, math::point, math::Point, path::Path, FillOptions,
    FillTessellator, VertexBuffers,
};

use crate::command::TurtleDrawState;

#[derive(Clone, Debug)]
pub struct TurtlePolygon {
    vertices: Vec<[f64; 2]>,
    indices: Vec<usize>,
}

impl TurtlePolygon {
    pub fn new(diagram: &[[f32; 2]]) -> Self {
        let mut path_builder = Path::builder();
        let mut iter = diagram.iter();

        println!("our coords: {diagram:?}");

        let first = iter.next().unwrap();
        path_builder.begin(point(first[0], first[1]));
        for i in iter {
            path_builder.line_to(point(i[0], i[1]));
        }
        path_builder.end(true);
        let path = path_builder.build();

        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

        {
            let mut vertex_builder = simple_builder(&mut buffers);

            // Create the tessellator.
            let mut tessellator = FillTessellator::new();

            // Compute the tessellation.
            let result =
                tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
            assert!(result.is_ok());
        }

        Self {
            vertices: buffers
                .vertices
                .into_iter()
                .map(|v| [v.x as f64, v.y as f64])
                .collect(),
            indices: buffers.indices.into_iter().map(|i| i as usize).collect(),
        }
    }

    pub fn draw(&self, color: &[f32; 4], transform: &[[f64; 3]; 2], ds: &mut TurtleDrawState) {
        for i in self.indices.chunks(3) {
            let shape = [
                self.vertices[i[0]],
                self.vertices[i[1]],
                self.vertices[i[2]],
            ];

            graphics::polygon(*color, &shape, *transform, ds.gl);
        }
    }
}
