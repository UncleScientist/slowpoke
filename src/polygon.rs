use std::collections::HashMap;

use iced::widget::canvas::Frame;
use iced::widget::canvas::Path as IPath;
use lyon_tessellation::{
    geometry_builder::simple_builder, math::point, math::Point, path::Path, FillOptions,
    FillTessellator, VertexBuffers,
};
use opengl_graphics::GlGraphics;

use crate::color_names::TurtleColor;

const CLASSIC: [[f32; 2]; 5] = [[0., 0.], [-15., 6.], [-10., 0.], [-15., -6.], [0., 0.]];
const ARROW: [[f32; 2]; 4] = [[0., 0.], [-10., 12.], [-10., -12.], [0., 0.]];
const CIRCLE: [[f32; 2]; 10] = [
    [0., 0.],
    [0., 3.],
    [-5., 9.],
    [-13., 9.],
    [-18., 3.],
    [-18., -3.],
    [-13., -9.],
    [-5., -9.],
    [0., -3.],
    [0., 0.],
];
const SQUARE: [[f32; 2]; 6] = [
    [0., 0.],
    [0., 8.],
    [-16., 8.],
    [-16., -8.],
    [0., -8.],
    [0., 0.],
];
const TRIANGLE: [[f32; 2]; 4] = [[0., 0.], [-16., 8.], [-16., -8.], [0., 0.]];

// TODO: add complex turtle shape

const SHAPES: [(&str, &[[f32; 2]]); 5] = [
    ("classic", &CLASSIC),
    ("arrow", &ARROW),
    ("circle", &CIRCLE),
    ("square", &SQUARE),
    ("triangle", &TRIANGLE),
];

#[derive(Debug, Clone)]
pub struct TurtleShape {
    pub(crate) name: String,
    pub(crate) shape: TurtlePolygon,
}

impl Default for TurtleShape {
    fn default() -> Self {
        Self {
            name: "classic".into(),
            shape: TurtlePolygon::new(&CLASSIC),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TurtleShapeName {
    GetCurrent,
    Shape(String),
}

impl From<&str> for TurtleShapeName {
    fn from(name: &str) -> Self {
        Self::Shape(name.into())
    }
}

#[derive(Clone, Debug)]
pub struct TurtlePolygon {
    ipath: IPath,
    // vertices: Vec<[f64; 2]>,
    // indices: Vec<usize>,
}

impl TurtlePolygon {
    pub fn new(diagram: &[[f32; 2]]) -> Self {
        let mut path_builder = Path::builder();
        let mut iter = diagram.iter();

        let first = iter.next().unwrap();
        let ipath = IPath::new(|b| {
            b.move_to((*first).into());
            for i in iter {
                b.line_to((*i).into());
            }
        });
        /*
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
        */

        Self {
            ipath,
            /*
            vertices: buffers
                .vertices
                .into_iter()
                .map(|v| [v.x as f64, v.y as f64])
                .collect(),
            indices: buffers.indices.into_iter().map(|i| i as usize).collect(),
            */
        }
    }

    pub fn draw(&self, color: &TurtleColor, transform: [[f64; 3]; 2], gl: &mut GlGraphics) {
        /*
        let color: [f32; 4] = (*color).into();
        for i in self.indices.chunks(3) {
            let shape = [
                self.vertices[i[0]],
                self.vertices[i[1]],
                self.vertices[i[2]],
            ];

            graphics::polygon(color, &shape, transform, gl);
        }
        */
    }

    pub(crate) fn get_path(&self) -> &IPath {
        &self.ipath
    }
}

pub(crate) fn generate_default_shapes() -> HashMap<String, TurtleShape> {
    let mut shapes = HashMap::new();

    for (name, poly) in &SHAPES {
        shapes.insert(
            (*name).into(),
            TurtleShape {
                name: (*name).into(),
                shape: TurtlePolygon::new(poly),
            },
        );
    }

    shapes
}
